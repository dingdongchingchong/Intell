//! Integration tests — require DATABASE_URL and a running Postgres.
//! Run: `docker compose up -d && cargo test --test api_integration -- --ignored`

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use caseflow_cms::config::Settings;
use caseflow_cms::db::create_pool;
use caseflow_cms::state::AppState;
use caseflow_cms::{build_app, seed_admin};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn app() -> axum::Router {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env().expect("settings");
    let pool = create_pool(&settings).await.expect("pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");
    let state = AppState::new(pool, settings);
    seed_admin(&state).await.expect("seed");
    build_app(state).await.expect("app")
}

#[tokio::test]
#[ignore = "requires postgres"]
async fn health_ok() {
    let app = app().await;
    let res = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires postgres"]
async fn register_login_flow() {
    let app = Arc::new(app().await);
    let email = format!("user_{}@test.local", uuid::Uuid::new_v4());
    let body = serde_json::json!({
        "email": email,
        "username": format!("u{}", &uuid::Uuid::new_v4().to_string()[..8]),
        "password": "password1",
        "display_name": "Test User"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["data"]["tokens"]["access_token"].as_str().is_some());

    let login = serde_json::json!({ "login": email, "password": "password1" });
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
