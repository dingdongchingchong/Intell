use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::category::Category;

pub struct CategoryRepo;

impl CategoryRepo {
    pub async fn create(db: &PgPool, name: &str, slug: &str, description: &str) -> AppResult<Category> {
        Ok(sqlx::query_as::<_, Category>(
            "INSERT INTO categories (name, slug, description) VALUES ($1,$2,$3) RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .bind(description)
        .fetch_one(db)
        .await?)
    }

    pub async fn list(db: &PgPool) -> AppResult<Vec<Category>> {
        Ok(sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY name")
            .fetch_all(db)
            .await?)
    }

    pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<Category>> {
        Ok(
            sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
                .bind(id)
                .fetch_optional(db)
                .await?,
        )
    }

    pub async fn delete(db: &PgPool, id: Uuid) -> AppResult<bool> {
        Ok(sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected()
            > 0)
    }
}
