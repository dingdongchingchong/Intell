use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::dto::common::{Paginated, PaginationQuery};
use crate::dto::users::{AdminUpdateUserRequest, UserListItem};
use crate::error::{AppError, AppResult};
use crate::models::post::PostStatus;
use crate::models::user::{PublicUser, User, UserRole};
use crate::repositories::{PostRepo, UserRepo};
use crate::services::auth::AuthService;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminDashboard {
    pub users_total: i64,
    pub posts_by_status: Vec<StatusCount>,
    pub published_posts: i64,
    pub pending_review: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusCount {
    pub status: PostStatus,
    pub count: i64,
}

pub struct AdminService;

impl AdminService {
    pub async fn dashboard(state: &AppState) -> AppResult<AdminDashboard> {
        let (_, users_total) = UserRepo::list(&state.db, 1, 0).await?;
        let counts = PostRepo::count_by_status(&state.db).await?;
        let published = counts
            .iter()
            .find(|(s, _)| matches!(s, PostStatus::Published))
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let pending = counts
            .iter()
            .find(|(s, _)| matches!(s, PostStatus::PendingReview))
            .map(|(_, c)| *c)
            .unwrap_or(0);

        Ok(AdminDashboard {
            users_total,
            posts_by_status: counts
                .into_iter()
                .map(|(status, count)| StatusCount { status, count })
                .collect(),
            published_posts: published,
            pending_review: pending,
        })
    }

    pub async fn list_users(
        state: &AppState,
        q: PaginationQuery,
    ) -> AppResult<Paginated<UserListItem>> {
        let (users, total) = UserRepo::list(&state.db, q.limit(), q.offset()).await?;
        let items = users
            .into_iter()
            .map(|u| UserListItem {
                email: u.email.clone(),
                is_active: u.is_active,
                is_verified: u.is_verified,
                last_login_at: u.last_login_at,
                user: PublicUser::from(u),
            })
            .collect();
        Ok(Paginated {
            items,
            page: q.page.max(1),
            per_page: q.per_page.clamp(1, 100),
            total,
        })
    }

    pub async fn update_user(
        state: &AppState,
        actor: &User,
        target_id: Uuid,
        req: AdminUpdateUserRequest,
    ) -> AppResult<PublicUser> {
        if !actor.role.can_manage_users() {
            return Err(AppError::Forbidden("admin only".into()));
        }
        let target = UserRepo::find_by_id(&state.db, target_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        if target.role == UserRole::Admin
            && (req.is_active == Some(false)
                || req.role.is_some_and(|r| r != UserRole::Admin))
        {
            let admins = UserRepo::count_admins(&state.db).await?;
            if admins <= 1 {
                return Err(AppError::Conflict(
                    "cannot demote or disable the last admin".into(),
                ));
            }
        }

        let password_hash = if let Some(pw) = &req.password {
            if pw.len() < 6 {
                return Err(AppError::Validation(
                    "password must be at least 6 characters".into(),
                ));
            }
            Some(AuthService::hash_password(pw)?)
        } else {
            None
        };

        let updated = UserRepo::admin_update(
            &state.db,
            target_id,
            req.role,
            req.is_active,
            req.display_name.as_deref(),
            password_hash.as_deref(),
        )
        .await?;

        sqlx::query(
            r#"INSERT INTO audit_logs (actor_id, action, entity_type, entity_id, metadata)
               VALUES ($1, 'admin_update_user', 'user', $2, $3)"#,
        )
        .bind(actor.id)
        .bind(target_id)
        .bind(serde_json::json!({ "role": req.role, "is_active": req.is_active }))
        .execute(&state.db)
        .await?;

        Ok(PublicUser::from(updated))
    }

    pub async fn moderate_post(
        state: &AppState,
        actor: &User,
        post_id: Uuid,
        status: PostStatus,
    ) -> AppResult<crate::models::post::Post> {
        if !actor.role.can_moderate() {
            return Err(AppError::Forbidden("moderation requires editor/admin".into()));
        }
        let post = PostRepo::update(&state.db, post_id, None, None, None, None, None, Some(status))
            .await?;
        sqlx::query(
            r#"INSERT INTO audit_logs (actor_id, action, entity_type, entity_id, metadata)
               VALUES ($1, 'moderate_post', 'post', $2, $3)"#,
        )
        .bind(actor.id)
        .bind(post_id)
        .bind(serde_json::json!({ "status": status }))
        .execute(&state.db)
        .await?;
        Ok(post)
    }

    pub async fn list_posts(
        state: &AppState,
        q: PaginationQuery,
        status: Option<PostStatus>,
    ) -> AppResult<Paginated<crate::models::post::Post>> {
        let (items, total) =
            PostRepo::list_all_admin(&state.db, q.limit(), q.offset(), status).await?;
        Ok(Paginated {
            items,
            page: q.page.max(1),
            per_page: q.per_page.clamp(1, 100),
            total,
        })
    }
}
