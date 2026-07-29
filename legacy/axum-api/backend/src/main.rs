use std::net::SocketAddr;
use std::time::Duration;

use caseflow_cms::config::Settings;
use caseflow_cms::db::create_pool;
use caseflow_cms::state::AppState;
use caseflow_cms::{build_app, seed_admin};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::from_env()?;
    settings.validate_bind_security()?;

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&settings.rust_log)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        app = %settings.app_name,
        env = %settings.app_env,
        host = %settings.app_host,
        port = settings.app_port,
        allowlist = ?settings.allowed_cidrs,
        "starting CaseFlow CMS"
    );

    if settings.app_host == "0.0.0.0" || settings.app_host == "::" {
        tracing::warn!(
            "APP_HOST binds all interfaces — ensure firewall/VPN restricts access to private networks only"
        );
    }

    let pool = create_pool(&settings).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("database migrations applied");

    let state = AppState::new(pool, settings.clone());
    seed_admin(&state).await?;

    let app = build_app(state).await?;
    let addr: SocketAddr = format!("{}:{}", settings.app_host, settings.app_port).parse()?;

    tracing::info!(%addr, "listening");
    tracing::info!("swagger ui: http://{}/swagger-ui/", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
    tokio::time::sleep(Duration::from_millis(100)).await;
}
