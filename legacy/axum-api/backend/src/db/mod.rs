use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Settings;
use crate::error::AppResult;

pub async fn create_pool(settings: &Settings) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.database_max_connections)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&settings.database_url)
        .await?;
    Ok(pool)
}
