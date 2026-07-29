use caseflow_api::{boot, cors_preflight, json_err, json_ok, parse_json, require_user, user_id};
use caseflow_core::auth::{authorize, Permission};
use caseflow_core::models::CreateUserRequest;
use caseflow_core::services::users as users_svc;
use http::Method;
use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    if req.method() == Method::OPTIONS {
        return cors_preflight();
    }
    match dispatch(req).await {
        Ok(v) => json_ok(v),
        Err(e) => json_err(e),
    }
}

async fn dispatch(req: Request) -> caseflow_core::AppResult<serde_json::Value> {
    let (settings, pool) = boot().await?;
    let claims = require_user(req.headers(), &settings)?;
    let actor = user_id(&claims)?;

    match *req.method() {
        Method::GET => {
            authorize(&claims.role, Permission::UserRead)?;
            let users = users_svc::list_users(&pool).await?;
            Ok(json!({ "users": users }))
        }
        Method::POST => {
            authorize(&claims.role, Permission::UserCreate)?;
            let body: CreateUserRequest = parse_json(req.body())?;
            let user = users_svc::create_user(&pool, actor, body).await?;
            Ok(json!({ "user": user }))
        }
        _ => Err(caseflow_core::AppError::Validation("method not allowed".into())),
    }
}
