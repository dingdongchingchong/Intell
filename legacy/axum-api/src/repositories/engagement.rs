use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::engagement::{Bookmark, Follow, Like, Share};

pub struct EngagementRepo;

impl EngagementRepo {
    pub async fn like_post(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let mut tx = db.begin().await?;
        let res = sqlx::query(
            "INSERT INTO likes (user_id, post_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(post_id)
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() > 0 {
            sqlx::query("UPDATE posts SET like_count = like_count + 1 WHERE id = $1")
                .bind(post_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(true);
        }
        tx.rollback().await?;
        Ok(false)
    }

    pub async fn unlike_post(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let mut tx = db.begin().await?;
        let res = sqlx::query("DELETE FROM likes WHERE user_id = $1 AND post_id = $2")
            .bind(user_id)
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
        if res.rows_affected() > 0 {
            sqlx::query(
                "UPDATE posts SET like_count = GREATEST(like_count - 1, 0) WHERE id = $1",
            )
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(true);
        }
        tx.rollback().await?;
        Ok(false)
    }

    pub async fn has_liked_post(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND post_id = $2)",
        )
        .bind(user_id)
        .bind(post_id)
        .fetch_one(db)
        .await?;
        Ok(exists)
    }

    pub async fn bookmark(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let mut tx = db.begin().await?;
        let res = sqlx::query(
            "INSERT INTO bookmarks (user_id, post_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(post_id)
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() > 0 {
            sqlx::query("UPDATE posts SET bookmark_count = bookmark_count + 1 WHERE id = $1")
                .bind(post_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(true);
        }
        tx.rollback().await?;
        Ok(false)
    }

    pub async fn unbookmark(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let mut tx = db.begin().await?;
        let res = sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 AND post_id = $2")
            .bind(user_id)
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
        if res.rows_affected() > 0 {
            sqlx::query(
                "UPDATE posts SET bookmark_count = GREATEST(bookmark_count - 1, 0) WHERE id = $1",
            )
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(true);
        }
        tx.rollback().await?;
        Ok(false)
    }

    pub async fn has_bookmarked(db: &PgPool, user_id: Uuid, post_id: Uuid) -> AppResult<bool> {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND post_id = $2)",
        )
        .bind(user_id)
        .bind(post_id)
        .fetch_one(db)
        .await?;
        Ok(exists)
    }

    pub async fn share(db: &PgPool, user_id: Uuid, post_id: Uuid, platform: &str) -> AppResult<Share> {
        let mut tx = db.begin().await?;
        let share = sqlx::query_as::<_, Share>(
            "INSERT INTO shares (user_id, post_id, platform) VALUES ($1,$2,$3) RETURNING *",
        )
        .bind(user_id)
        .bind(post_id)
        .bind(platform)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("UPDATE posts SET share_count = share_count + 1 WHERE id = $1")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(share)
    }

    pub async fn follow(db: &PgPool, follower: Uuid, following: Uuid) -> AppResult<bool> {
        if follower == following {
            return Ok(false);
        }
        let res = sqlx::query(
            "INSERT INTO follows (follower_id, following_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(follower)
        .bind(following)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn unfollow(db: &PgPool, follower: Uuid, following: Uuid) -> AppResult<bool> {
        let res = sqlx::query(
            "DELETE FROM follows WHERE follower_id = $1 AND following_id = $2",
        )
        .bind(follower)
        .bind(following)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn follow_stats(db: &PgPool, user_id: Uuid) -> AppResult<(i64, i64)> {
        let (followers,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM follows WHERE following_id = $1")
                .bind(user_id)
                .fetch_one(db)
                .await?;
        let (following,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM follows WHERE follower_id = $1")
                .bind(user_id)
                .fetch_one(db)
                .await?;
        Ok((followers, following))
    }

    pub async fn list_bookmarks(
        db: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Bookmark>> {
        Ok(sqlx::query_as::<_, Bookmark>(
            "SELECT * FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?)
    }

    #[allow(dead_code)]
    pub async fn list_likes_for_user(db: &PgPool, user_id: Uuid) -> AppResult<Vec<Like>> {
        Ok(sqlx::query_as::<_, Like>(
            "SELECT * FROM likes WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(db)
        .await?)
    }

    #[allow(dead_code)]
    pub async fn list_follows(db: &PgPool, user_id: Uuid) -> AppResult<Vec<Follow>> {
        Ok(sqlx::query_as::<_, Follow>(
            "SELECT * FROM follows WHERE follower_id = $1",
        )
        .bind(user_id)
        .fetch_all(db)
        .await?)
    }
}
