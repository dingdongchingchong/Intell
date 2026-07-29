use caseflow_api::{boot, cors_preflight, json_err, json_ok, parse_json};
use caseflow_core::models::LoginRequest;
use caseflow_core::services::auth as auth_svc;
use http::Method;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    if req.method() == Method::OPTIONS {
        return cors_preflight();
    }
    if req.method() != Method::POST {
        return json_err(caseflow_core::AppError::Validation("POST required".into()));
    }

    match inner(req).await {
        Ok(v) => json_ok(v),
        Err(e) => json_err(e),
    }
}

async fn inner(req: Request) -> caseflow_core::AppResult<caseflow_core::models::LoginResponse> {
    let (settings, pool) = boot().await?;
    let body = req.body();
    let login: LoginRequest = parse_json(body)?;
    auth_svc::login(&pool, &settings, login).await
}
