use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

use crate::dto::notifications::{WsClientMessage, WsServerMessage};
use crate::error::AppError;
use crate::services::auth::AuthService;
use crate::services::notifications::NotificationService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let claims = AuthService::decode_access_token(&state, &q.token)?;
    let user_id = claims.sub;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: uuid::Uuid) {
    *state.ws_connections.entry(user_id).or_insert(0) += 1;
    tracing::info!(%user_id, "websocket connected");

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.notifications.subscribe();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) if event.user_id == user_id => {
                    let msg = WsServerMessage::notification(&event.notification);
                    if let Ok(text) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                }
                Ok(_) => continue,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client {
                        WsClientMessage::Ping => {
                            // handled by client; server may reply via separate channel if needed
                        }
                        WsClientMessage::MarkRead { notification_id } => {
                            let _ =
                                NotificationService::mark_read(&state, user_id, notification_id)
                                    .await;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    if let Some(mut entry) = state.ws_connections.get_mut(&user_id) {
        *entry = entry.saturating_sub(1);
    }
    tracing::info!(%user_id, "websocket disconnected");
}
