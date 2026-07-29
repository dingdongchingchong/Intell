use caseflow_api::{boot, cors_preflight, json_err, json_ok, require_user};
use caseflow_core::auth::{authorize, Permission};
use caseflow_core::services::cases as cases_svc;
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
    if req.method() != Method::GET {
        return json_err(caseflow_core::AppError::Validation("GET required".into()));
    }
    match inner(req).await {
        Ok(v) => json_ok(v),
        Err(e) => json_err(e),
    }
}

async fn inner(req: Request) -> caseflow_core::AppResult<serde_json::Value> {
    let (settings, pool) = boot().await?;
    let claims = require_user(req.headers(), &settings)?;
    authorize(&claims.role, Permission::CaseRead)?;
    let stats = cases_svc::dashboard_stats(&pool).await?;
    Ok(json!({ "stats": stats }))
}
