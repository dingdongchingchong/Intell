use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Like,
    Comment,
    Follow,
    Mention,
    Share,
    Bookmark,
    System,
    Moderation,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub r#type: NotificationType,
    pub title: String,
    pub body: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}
