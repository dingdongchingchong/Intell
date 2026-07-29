use std::sync::Arc;

use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::config::Settings;
use crate::dto::notifications::NotificationEvent;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub settings: Arc<Settings>,
    pub notifications: broadcast::Sender<NotificationEvent>,
    /// user_id -> open websocket connection count (for presence / metrics)
    pub ws_connections: Arc<DashMap<Uuid, usize>>,
}

impl AppState {
    pub fn new(db: PgPool, settings: Settings) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            db,
            settings: Arc::new(settings),
            notifications: tx,
            ws_connections: Arc::new(DashMap::new()),
        }
    }
}
