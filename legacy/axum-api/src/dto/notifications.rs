use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::notification::{Notification, NotificationType};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationEvent {
    pub user_id: Uuid,
    pub notification: Notification,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationList {
    pub items: Vec<Notification>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WsServerMessage {
    pub event: String,
    pub payload: serde_json::Value,
}

impl WsServerMessage {
    pub fn notification(n: &Notification) -> Self {
        Self {
            event: "notification".into(),
            payload: serde_json::to_value(n).unwrap_or_default(),
        }
    }

    pub fn pong() -> Self {
        Self {
            event: "pong".into(),
            payload: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Ping,
    MarkRead { notification_id: Uuid },
}

#[allow(dead_code)]
pub fn notif_type_label(t: NotificationType) -> &'static str {
    match t {
        NotificationType::Like => "like",
        NotificationType::Comment => "comment",
        NotificationType::Follow => "follow",
        NotificationType::Mention => "mention",
        NotificationType::Share => "share",
        NotificationType::Bookmark => "bookmark",
        NotificationType::System => "system",
        NotificationType::Moderation => "moderation",
    }
}
