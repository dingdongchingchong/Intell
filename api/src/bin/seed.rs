//! Seed admin user and run migrations.
//! Usage: `cargo run -p caseflow-api --bin seed`

use caseflow_core::config::Settings;
use caseflow_core::db;
use caseflow_core::services::auth as auth_svc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env()?;
    let pool = db::create_pool(&settings).await?;
    db::migrate(&pool).await?;
    match auth_svc::seed_admin(&pool, &settings).await? {
        Some(id) => println!("Seeded admin {} ({})", settings.seed_admin_username, id),
        None => println!("Users already exist — skipped seed"),
    }
    Ok(())
}
