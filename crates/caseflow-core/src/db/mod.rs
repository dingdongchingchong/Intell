use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::Settings;
use crate::error::AppResult;

pub async fn create_pool(settings: &Settings) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&settings.database_url)
        .await?;
    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| crate::error::AppError::Other(anyhow::anyhow!(e)))?;
    Ok(())
}
