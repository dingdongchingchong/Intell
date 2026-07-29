use caseflow_api::{
    boot, cors_preflight, json_err, json_ok, parse_json, path_uuid, query_param, require_user,
    user_id,
};
use caseflow_core::auth::{authorize, Permission};
use caseflow_core::models::{CaseListQuery, CreateCaseRequest, UpdateCaseRequest, UpdateStageRequest};
use caseflow_core::services::cases as cases_svc;
use http::Method;
use serde_json::json;
use uuid::Uuid;
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
    let path = req.uri().path();
    let method = req.method().clone();

    // /api/v1/cases/:id/stage
    if path.contains("/stage") && method == Method::PATCH {
        authorize(&claims.role, Permission::CaseUpdate)?;
        let id = extract_case_id(path)?;
        let body: UpdateStageRequest = parse_json(req.body())?;
        let case = cases_svc::update_stage(&pool, actor, id, body).await?;
        return Ok(json!({ "case": case }));
    }

    // /api/v1/cases/:id
    if let Ok(id) = extract_case_id(path) {
        match method {
            Method::GET => {
                authorize(&claims.role, Permission::CaseRead)?;
                let case = cases_svc::get_case(&pool, id).await?;
                Ok(json!({ "case": case }))
            }
            Method::PUT | Method::PATCH => {
                authorize(&claims.role, Permission::CaseUpdate)?;
                let body: UpdateCaseRequest = parse_json(req.body())?;
                let case = cases_svc::update_case(&pool, actor, id, body).await?;
                Ok(json!({ "case": case }))
            }
            Method::DELETE => {
                authorize(&claims.role, Permission::CaseDelete)?;
                cases_svc::soft_delete(&pool, actor, id).await?;
                Ok(json!({ "ok": true }))
            }
            _ => Err(caseflow_core::AppError::Validation("method not allowed".into())),
        }
    } else {
        match method {
            Method::GET => {
                authorize(&claims.role, Permission::CaseRead)?;
                if path.ends_with("/clients") || path.contains("/clients") {
                    let clients = cases_svc::list_clients(&pool).await?;
                    return Ok(json!({ "clients": clients }));
                }
                if path.ends_with("/next-id") {
                    let next = cases_svc::next_case_number(&pool).await?;
                    return Ok(json!({ "case_number": next }));
                }
                let q = CaseListQuery {
                    search: query_param(&req, "search").map(|s| s.to_string()),
                    stage: query_param(&req, "stage").map(|s| s.to_string()),
                    investigator_id: query_param(&req, "investigator_id")
                        .and_then(|s| Uuid::parse_str(s).ok()),
                    priority: query_param(&req, "priority").map(|s| s.to_string()),
                    page: query_param(&req, "page").and_then(|s| s.parse().ok()),
                    page_size: query_param(&req, "page_size").and_then(|s| s.parse().ok()),
                };
                let (cases, total) = cases_svc::list_cases(&pool, q).await?;
                Ok(json!({ "cases": cases, "total": total }))
            }
            Method::POST => {
                authorize(&claims.role, Permission::CaseCreate)?;
                let body: CreateCaseRequest = parse_json(req.body())?;
                let case = cases_svc::create_case(&pool, actor, body).await?;
                Ok(json!({ "case": case }))
            }
            _ => Err(caseflow_core::AppError::Validation("method not allowed".into())),
        }
    }
}

fn extract_case_id(path: &str) -> caseflow_core::AppResult<Uuid> {
    // Accept .../cases/<uuid> or .../cases/<uuid>/stage
    let marker = "/cases/";
    let idx = path
        .find(marker)
        .ok_or_else(|| caseflow_core::AppError::Validation("missing case id".into()))?;
    let rest = &path[idx + marker.len()..];
    let id_str = rest.split('/').next().unwrap_or("");
    if id_str.is_empty() || id_str == "clients" || id_str == "next-id" {
        return Err(caseflow_core::AppError::Validation("missing case id".into()));
    }
    Uuid::parse_str(id_str).map_err(|_| caseflow_core::AppError::Validation("invalid uuid".into()))
}

#[allow(dead_code)]
fn _path_uuid_compat(path: &str) -> caseflow_core::AppResult<Uuid> {
    path_uuid(path, "/api/v1/cases/")
}
