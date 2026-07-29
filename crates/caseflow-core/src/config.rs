use std::env;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_ttl_secs: i64,
    pub seed_admin_email: String,
    pub seed_admin_username: String,
    pub seed_admin_password: String,
    pub seed_admin_name: String,
}

impl Settings {
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt_secret: required("JWT_SECRET")?,
            jwt_issuer: env_or("JWT_ISSUER", "caseflow"),
            jwt_ttl_secs: env_or("JWT_TTL_SECS", "86400").parse().unwrap_or(86_400),
            seed_admin_email: env_or("SEED_ADMIN_EMAIL", "admin@caseflow.local"),
            seed_admin_username: env_or("SEED_ADMIN_USERNAME", "admin"),
            seed_admin_password: env_or("SEED_ADMIN_PASSWORD", "admin123456"),
            seed_admin_name: env_or("SEED_ADMIN_NAME", "Administrator"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn required(key: &str) -> AppResult<String> {
    env::var(key).map_err(|_| AppError::Config(format!("{key} must be set")))
}
