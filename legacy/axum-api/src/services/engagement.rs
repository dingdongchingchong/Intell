use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::notification::NotificationType;
use crate::models::user::User;
use crate::repositories::{EngagementRepo, PostRepo};
use crate::services::notifications::NotificationService;
use crate::state::AppState;

pub struct EngagementService;

impl EngagementService {
    pub async fn like_post(state: &AppState, user: &User, post_id: Uuid) -> AppResult<()> {
        let post = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;
        let created = EngagementRepo::like_post(&state.db, user.id, post_id).await?;
        if created && post.author_id != user.id {
            NotificationService::notify(
                state,
                post.author_id,
                Some(user.id),
                NotificationType::Like,
                "New like",
                &format!("{} liked your post", user.username),
                Some("post"),
                Some(post_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn unlike_post(state: &AppState, user_id: Uuid, post_id: Uuid) -> AppResult<()> {
        EngagementRepo::unlike_post(&state.db, user_id, post_id).await?;
        Ok(())
    }

    pub async fn bookmark(state: &AppState, user: &User, post_id: Uuid) -> AppResult<()> {
        let post = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;
        let created = EngagementRepo::bookmark(&state.db, user.id, post_id).await?;
        if created && post.author_id != user.id {
            NotificationService::notify(
                state,
                post.author_id,
                Some(user.id),
                NotificationType::Bookmark,
                "New bookmark",
                &format!("{} bookmarked your post", user.username),
                Some("post"),
                Some(post_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn unbookmark(state: &AppState, user_id: Uuid, post_id: Uuid) -> AppResult<()> {
        EngagementRepo::unbookmark(&state.db, user_id, post_id).await?;
        Ok(())
    }

    pub async fn share(
        state: &AppState,
        user: &User,
        post_id: Uuid,
        platform: &str,
    ) -> AppResult<()> {
        let post = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;
        EngagementRepo::share(&state.db, user.id, post_id, platform).await?;
        if post.author_id != user.id {
            NotificationService::notify(
                state,
                post.author_id,
                Some(user.id),
                NotificationType::Share,
                "Post shared",
                &format!("{} shared your post on {platform}", user.username),
                Some("post"),
                Some(post_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn follow(state: &AppState, follower: &User, following_id: Uuid) -> AppResult<()> {
        if follower.id == following_id {
            return Err(AppError::BadRequest("cannot follow yourself".into()));
        }
        let _ = crate::repositories::UserRepo::find_by_id(&state.db, following_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        let created = EngagementRepo::follow(&state.db, follower.id, following_id).await?;
        if created {
            NotificationService::notify(
                state,
                following_id,
                Some(follower.id),
                NotificationType::Follow,
                "New follower",
                &format!("{} started following you", follower.username),
                Some("user"),
                Some(follower.id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn unfollow(state: &AppState, follower_id: Uuid, following_id: Uuid) -> AppResult<()> {
        EngagementRepo::unfollow(&state.db, follower_id, following_id).await?;
        Ok(())
    }
}
