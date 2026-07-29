use slug::slugify;
use uuid::Uuid;

use crate::dto::common::Paginated;
use crate::dto::posts::{CreatePostRequest, PostDetail, UpdatePostRequest};
use crate::error::{AppError, AppResult};
use crate::models::notification::NotificationType;
use crate::models::post::{Post, PostStatus};
use crate::models::user::{User, UserRole};
use crate::repositories::{
    EngagementRepo, PostRepo, TagRepo, UserRepo,
};
use crate::services::notifications::NotificationService;
use crate::state::AppState;

pub struct ContentService;

impl ContentService {
    pub async fn create_post(
        state: &AppState,
        author: &User,
        req: CreatePostRequest,
    ) -> AppResult<Post> {
        if !author.role.can_publish() {
            return Err(AppError::Forbidden("insufficient role to create posts".into()));
        }
        let status = req.status.unwrap_or(PostStatus::Draft);
        if matches!(status, PostStatus::Published) && !author.role.can_publish() {
            return Err(AppError::Forbidden("cannot publish".into()));
        }

        let base_slug = slugify(&req.title);
        let mut slug = base_slug.clone();
        let mut i = 1;
        while PostRepo::find_by_slug(&state.db, &slug).await?.is_some() {
            i += 1;
            slug = format!("{base_slug}-{i}");
        }

        let excerpt = req
            .excerpt
            .unwrap_or_else(|| req.content.chars().take(180).collect());

        let post = PostRepo::create(
            &state.db,
            author.id,
            req.category_id,
            &req.title,
            &slug,
            &excerpt,
            &req.content,
            req.cover_image_url.as_deref(),
            status,
        )
        .await?;

        if let Some(names) = req.tag_names {
            let mut ids = Vec::new();
            for name in names {
                let s = slugify(&name);
                let tag = TagRepo::find_or_create(&state.db, &name, &s).await?;
                ids.push(tag.id);
            }
            PostRepo::set_tags(&state.db, post.id, &ids).await?;
        }

        Ok(post)
    }

    pub async fn update_post(
        state: &AppState,
        actor: &User,
        post_id: Uuid,
        req: UpdatePostRequest,
    ) -> AppResult<Post> {
        let existing = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;

        if existing.author_id != actor.id && !actor.role.can_moderate() {
            return Err(AppError::Forbidden("cannot edit this post".into()));
        }

        let updated = PostRepo::update(
            &state.db,
            post_id,
            req.title.as_deref(),
            req.content.as_deref(),
            req.excerpt.as_deref(),
            req.category_id.map(Some),
            req.cover_image_url.as_deref(),
            req.status,
        )
        .await?;

        if let Some(names) = req.tag_names {
            let mut ids = Vec::new();
            for name in names {
                let s = slugify(&name);
                let tag = TagRepo::find_or_create(&state.db, &name, &s).await?;
                ids.push(tag.id);
            }
            PostRepo::set_tags(&state.db, post_id, &ids).await?;
        }

        Ok(updated)
    }

    pub async fn delete_post(state: &AppState, actor: &User, post_id: Uuid) -> AppResult<()> {
        let existing = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;
        if existing.author_id != actor.id && actor.role != UserRole::Admin {
            return Err(AppError::Forbidden("cannot delete this post".into()));
        }
        PostRepo::delete(&state.db, post_id).await?;
        Ok(())
    }

    pub async fn get_post(
        state: &AppState,
        slug_or_id: &str,
        viewer: Option<Uuid>,
    ) -> AppResult<PostDetail> {
        let post = if let Ok(id) = Uuid::parse_str(slug_or_id) {
            PostRepo::find_by_id(&state.db, id).await?
        } else {
            PostRepo::find_by_slug(&state.db, slug_or_id).await?
        }
        .ok_or_else(|| AppError::NotFound("post not found".into()))?;

        if !matches!(post.status, PostStatus::Published) {
            match viewer {
                Some(uid) if uid == post.author_id => {}
                Some(uid) => {
                    let u = UserRepo::find_by_id(&state.db, uid).await?;
                    if u.map(|u| u.role.can_moderate()).unwrap_or(false) {
                        // ok
                    } else {
                        return Err(AppError::NotFound("post not found".into()));
                    }
                }
                None => return Err(AppError::NotFound("post not found".into())),
            }
        }

        PostRepo::increment_view(&state.db, post.id).await?;
        let tags = TagRepo::for_post(&state.db, post.id).await?;
        let (liked, bookmarked) = if let Some(uid) = viewer {
            (
                EngagementRepo::has_liked_post(&state.db, uid, post.id).await?,
                EngagementRepo::has_bookmarked(&state.db, uid, post.id).await?,
            )
        } else {
            (false, false)
        };

        Ok(PostDetail {
            post,
            tags,
            liked_by_me: liked,
            bookmarked_by_me: bookmarked,
        })
    }

    pub async fn list_published(
        state: &AppState,
        page: u32,
        per_page: u32,
        category_id: Option<Uuid>,
        q: Option<&str>,
    ) -> AppResult<Paginated<Post>> {
        let limit = per_page.clamp(1, 100) as i64;
        let offset = ((page.max(1) - 1) as i64) * limit;
        let (items, total) =
            PostRepo::list_published(&state.db, limit, offset, category_id, q).await?;
        Ok(Paginated {
            items,
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            total,
        })
    }

    pub async fn add_comment(
        state: &AppState,
        author: &User,
        post_id: Uuid,
        body: &str,
        parent_id: Option<Uuid>,
    ) -> AppResult<crate::models::comment::Comment> {
        if body.trim().is_empty() {
            return Err(AppError::Validation("comment body required".into()));
        }
        let post = PostRepo::find_by_id(&state.db, post_id)
            .await?
            .ok_or_else(|| AppError::NotFound("post not found".into()))?;
        let comment =
            crate::repositories::CommentRepo::create(&state.db, post_id, author.id, parent_id, body)
                .await?;

        if post.author_id != author.id {
            NotificationService::notify(
                state,
                post.author_id,
                Some(author.id),
                NotificationType::Comment,
                "New comment",
                &format!("{} commented on your post", author.username),
                Some("post"),
                Some(post_id),
            )
            .await?;
        }
        Ok(comment)
    }
}
