use uuid::Uuid;

use crate::dto::notifications::{NotificationEvent, NotificationList};
use crate::error::AppResult;
use crate::models::notification::{Notification, NotificationType};
use crate::repositories::NotificationRepo;
use crate::state::AppState;

pub struct NotificationService;

impl NotificationService {
    pub async fn notify(
        state: &AppState,
        user_id: Uuid,
        actor_id: Option<Uuid>,
        r#type: NotificationType,
        title: &str,
        body: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AppResult<Notification> {
        let n = NotificationRepo::create(
            &state.db,
            user_id,
            actor_id,
            r#type,
            title,
            body,
            entity_type,
            entity_id,
        )
        .await?;

        let _ = state.notifications.send(NotificationEvent {
            user_id,
            notification: n.clone(),
        });

        Ok(n)
    }

    pub async fn list(
        state: &AppState,
        user_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> AppResult<NotificationList> {
        let limit = per_page.clamp(1, 100) as i64;
        let offset = ((page.max(1) - 1) as i64) * limit;
        let (items, unread_count) =
            NotificationRepo::list_for_user(&state.db, user_id, limit, offset).await?;
        Ok(NotificationList {
            items,
            unread_count,
        })
    }

    pub async fn mark_read(state: &AppState, user_id: Uuid, id: Uuid) -> AppResult<()> {
        NotificationRepo::mark_read(&state.db, user_id, id).await?;
        Ok(())
    }

    pub async fn mark_all_read(state: &AppState, user_id: Uuid) -> AppResult<u64> {
        NotificationRepo::mark_all_read(&state.db, user_id).await
    }
}
