use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::post::{Post, PostStatus};

pub struct PostRepo;

impl PostRepo {
    pub async fn create(
        db: &PgPool,
        author_id: Uuid,
        category_id: Option<Uuid>,
        title: &str,
        slug: &str,
        excerpt: &str,
        content: &str,
        cover_image_url: Option<&str>,
        status: PostStatus,
    ) -> AppResult<Post> {
        let published_at = if matches!(status, PostStatus::Published) {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let post = sqlx::query_as::<_, Post>(
            r#"INSERT INTO posts
               (author_id, category_id, title, slug, excerpt, content, cover_image_url, status, published_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING *"#,
        )
        .bind(author_id)
        .bind(category_id)
        .bind(title)
        .bind(slug)
        .bind(excerpt)
        .bind(content)
        .bind(cover_image_url)
        .bind(status)
        .bind(published_at)
        .fetch_one(db)
        .await?;
        Ok(post)
    }

    pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<Post>> {
        Ok(sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?)
    }

    pub async fn find_by_slug(db: &PgPool, slug: &str) -> AppResult<Option<Post>> {
        Ok(
            sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE slug = $1")
                .bind(slug)
                .fetch_optional(db)
                .await?,
        )
    }

    pub async fn update(
        db: &PgPool,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
        excerpt: Option<&str>,
        category_id: Option<Option<Uuid>>,
        cover_image_url: Option<&str>,
        status: Option<PostStatus>,
    ) -> AppResult<Post> {
        // Fetch current then apply patches for clarity
        let mut post = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("post not found".into()))?;

        if let Some(t) = title {
            post.title = t.to_string();
        }
        if let Some(c) = content {
            post.content = c.to_string();
        }
        if let Some(e) = excerpt {
            post.excerpt = e.to_string();
        }
        if let Some(cat) = category_id {
            post.category_id = cat;
        }
        if let Some(url) = cover_image_url {
            post.cover_image_url = Some(url.to_string());
        }
        if let Some(s) = status {
            if matches!(s, PostStatus::Published) && post.published_at.is_none() {
                post.published_at = Some(chrono::Utc::now());
            }
            post.status = s;
        }

        let updated = sqlx::query_as::<_, Post>(
            r#"UPDATE posts SET title=$2, content=$3, excerpt=$4, category_id=$5,
                 cover_image_url=$6, status=$7, published_at=$8, updated_at=NOW()
               WHERE id=$1 RETURNING *"#,
        )
        .bind(id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.excerpt)
        .bind(post.category_id)
        .bind(&post.cover_image_url)
        .bind(post.status)
        .bind(post.published_at)
        .fetch_one(db)
        .await?;
        Ok(updated)
    }

    pub async fn delete(db: &PgPool, id: Uuid) -> AppResult<bool> {
        let res = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn list_published(
        db: &PgPool,
        limit: i64,
        offset: i64,
        category_id: Option<Uuid>,
        q: Option<&str>,
    ) -> AppResult<(Vec<Post>, i64)> {
        let pattern = q.map(|s| format!("%{}%", s));
        let total: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM posts
               WHERE status = 'published'
                 AND ($1::uuid IS NULL OR category_id = $1)
                 AND ($2::text IS NULL OR title ILIKE $2 OR content ILIKE $2)"#,
        )
        .bind(category_id)
        .bind(&pattern)
        .fetch_one(db)
        .await?;

        let posts = sqlx::query_as::<_, Post>(
            r#"SELECT * FROM posts
               WHERE status = 'published'
                 AND ($1::uuid IS NULL OR category_id = $1)
                 AND ($2::text IS NULL OR title ILIKE $2 OR content ILIKE $2)
               ORDER BY published_at DESC NULLS LAST
               LIMIT $3 OFFSET $4"#,
        )
        .bind(category_id)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;
        Ok((posts, total.0))
    }

    pub async fn list_all_admin(
        db: &PgPool,
        limit: i64,
        offset: i64,
        status: Option<PostStatus>,
    ) -> AppResult<(Vec<Post>, i64)> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM posts WHERE ($1::post_status IS NULL OR status = $1)",
        )
        .bind(status)
        .fetch_one(db)
        .await?;
        let posts = sqlx::query_as::<_, Post>(
            r#"SELECT * FROM posts
               WHERE ($1::post_status IS NULL OR status = $1)
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;
        Ok((posts, total.0))
    }

    pub async fn increment_view(db: &PgPool, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE posts SET view_count = view_count + 1 WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn set_tags(db: &PgPool, post_id: Uuid, tag_ids: &[Uuid]) -> AppResult<()> {
        sqlx::query("DELETE FROM post_tags WHERE post_id = $1")
            .bind(post_id)
            .execute(db)
            .await?;
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(post_id)
                .bind(tag_id)
                .execute(db)
                .await?;
        }
        Ok(())
    }

    pub async fn count_by_status(db: &PgPool) -> AppResult<Vec<(PostStatus, i64)>> {
        let rows: Vec<(PostStatus, i64)> =
            sqlx::query_as("SELECT status, COUNT(*) FROM posts GROUP BY status")
                .fetch_all(db)
                .await?;
        Ok(rows)
    }
}
