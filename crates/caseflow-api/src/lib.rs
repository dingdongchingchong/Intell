//! Shared helpers for Vercel function binaries and the local Axum server.

use caseflow_core::auth::{validate_bearer, Claims};
use caseflow_core::config::Settings;
use caseflow_core::db;
use caseflow_core::error::AppError;
use caseflow_core::AppResult;
use http::HeaderMap;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use vercel_runtime::{Body, Response, StatusCode};

pub async fn boot() -> AppResult<(Settings, PgPool)> {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env()?;
    let pool = db::create_pool(&settings).await?;
    Ok((settings, pool))
}

pub fn require_user(headers: &HeaderMap, settings: &Settings) -> AppResult<Claims> {
    validate_bearer(headers, &settings.jwt_secret, &settings.jwt_issuer)
}

pub fn user_id(claims: &Claims) -> AppResult<Uuid> {
    Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("invalid subject in token".into()))
}

pub fn json_ok<T: Serialize>(value: T) -> Result<Response<Body>, vercel_runtime::Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "Authorization, Content-Type")
        .header("Access-Control-Allow-Methods", "GET,POST,PUT,PATCH,DELETE,OPTIONS")
        .body(serde_json::to_string(&value)?.into())?)
}

pub fn json_err(err: AppError) -> Result<Response<Body>, vercel_runtime::Error> {
    let body = json!({
        "error": err.code(),
        "message": err.to_string(),
    });
    Ok(Response::builder()
        .status(StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(body.to_string().into())?)
}

pub fn cors_preflight() -> Result<Response<Body>, vercel_runtime::Error> {
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "Authorization, Content-Type")
        .header("Access-Control-Allow-Methods", "GET,POST,PUT,PATCH,DELETE,OPTIONS")
        .body(Body::Empty)?)
}

pub fn parse_json<T: serde::de::DeserializeOwned>(body: &[u8]) -> AppResult<T> {
    serde_json::from_slice(body).map_err(|e| AppError::Validation(e.to_string()))
}

pub fn query_param<'a>(req: &'a vercel_runtime::Request, key: &str) -> Option<&'a str> {
    req.uri().query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let mut it = pair.splitn(2, '=');
            let k = it.next()?;
            let v = it.next().unwrap_or("");
            if k == key {
                Some(v)
            } else {
                None
            }
        })
    })
}

pub fn path_uuid(path: &str, prefix: &str) -> AppResult<Uuid> {
    let rest = path
        .strip_prefix(prefix)
        .ok_or_else(|| AppError::Validation("invalid path".into()))?;
    let id = rest.trim_matches('/').split('/').next().unwrap_or("");
    Uuid::parse_str(id).map_err(|_| AppError::Validation("invalid uuid".into()))
}

pub fn err_value(err: &AppError) -> Value {
    json!({ "error": err.code(), "message": err.to_string() })
}
