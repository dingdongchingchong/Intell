use std::env;

use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub app_name: String,
    pub app_env: String,
    pub app_host: String,
    pub app_port: u16,
    pub app_url: String,
    pub frontend_url: String,
    pub rust_log: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: i64,
    pub jwt_refresh_ttl_secs: i64,
    pub jwt_issuer: String,
    pub twitter_client_id: Option<String>,
    pub twitter_client_secret: Option<String>,
    pub twitter_redirect_uri: Option<String>,
    pub rate_limit_rps: u64,
    pub rate_limit_burst: u32,
    pub cors_origins: Vec<String>,
    /// Optional source CIDRs (e.g. VPN/LAN). Empty = disabled.
    pub allowed_cidrs: Vec<String>,
    /// Explicit opt-in to bind 0.0.0.0 in production (discouraged).
    pub allow_public_bind: bool,
    pub seed_admin_email: String,
    pub seed_admin_username: String,
    pub seed_admin_password: String,
    pub seed_admin_name: String,
}

impl Settings {
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            app_name: env_or("APP_NAME", "caseflow_cms"),
            app_env: env_or("APP_ENV", "development"),
            app_host: env_or("APP_HOST", "0.0.0.0"),
            app_port: env_or("APP_PORT", "8080")
                .parse()
                .map_err(|_| AppError::Config("APP_PORT must be a number".into()))?,
            app_url: env_or("APP_URL", "http://localhost:8080"),
            frontend_url: env_or("FRONTEND_URL", "http://localhost:3000"),
            rust_log: env_or("RUST_LOG", "caseflow_cms=debug,tower_http=info"),
            database_url: required("DATABASE_URL")?,
            database_max_connections: env_or("DATABASE_MAX_CONNECTIONS", "20")
                .parse()
                .unwrap_or(20),
            jwt_secret: required("JWT_SECRET")?,
            jwt_access_ttl_secs: env_or("JWT_ACCESS_TTL_SECS", "900")
                .parse()
                .unwrap_or(900),
            jwt_refresh_ttl_secs: env_or("JWT_REFRESH_TTL_SECS", "604800")
                .parse()
                .unwrap_or(604800),
            jwt_issuer: env_or("JWT_ISSUER", "caseflow_cms"),
            twitter_client_id: optional("TWITTER_CLIENT_ID"),
            twitter_client_secret: optional("TWITTER_CLIENT_SECRET"),
            twitter_redirect_uri: optional("TWITTER_REDIRECT_URI"),
            rate_limit_rps: env_or("RATE_LIMIT_RPS", "30").parse().unwrap_or(30),
            rate_limit_burst: env_or("RATE_LIMIT_BURST", "60").parse().unwrap_or(60),
            cors_origins: env_or(
                "CORS_ORIGINS",
                "http://localhost:3000,http://127.0.0.1:5500",
            )
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
            allowed_cidrs: env_or("ALLOWED_CIDRS", "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            allow_public_bind: matches!(
                env_or("ALLOW_PUBLIC_BIND", "false").to_ascii_lowercase().as_str(),
                "true" | "1" | "yes"
            ),
            seed_admin_email: env_or("SEED_ADMIN_EMAIL", "admin@caseflow.local"),
            seed_admin_username: env_or("SEED_ADMIN_USERNAME", "admin"),
            seed_admin_password: env_or("SEED_ADMIN_PASSWORD", "admin123456"),
            seed_admin_name: env_or("SEED_ADMIN_NAME", "Administrator"),
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }

    pub fn twitter_enabled(&self) -> bool {
        self.twitter_client_id
            .as_ref()
            .zip(self.twitter_client_secret.as_ref())
            .zip(self.twitter_redirect_uri.as_ref())
            .is_some()
    }

    /// Refuse insecure public binds in production unless explicitly allowed.
    pub fn validate_bind_security(&self) -> AppResult<()> {
        let publicish = self.app_host == "0.0.0.0" || self.app_host == "::";
        if self.is_production() && publicish && !self.allow_public_bind {
            return Err(AppError::Config(
                "production APP_HOST is 0.0.0.0/:: — bind a private IP (e.g. 192.168.x.x) \
                 or 127.0.0.1, or set ALLOW_PUBLIC_BIND=true only behind a locked-down firewall"
                    .into(),
            ));
        }
        if self.is_production() && self.jwt_secret.len() < 32 {
            return Err(AppError::Config(
                "JWT_SECRET must be at least 32 characters in production".into(),
            ));
        }
        // Validate CIDR syntax early
        crate::middleware::CidrAllowlistLayer::from_cidrs(&self.allowed_cidrs)
            .map_err(AppError::Config)?;
        Ok(())
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn optional(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.trim().is_empty())
}

fn required(key: &str) -> AppResult<String> {
    env::var(key).map_err(|_| AppError::Config(format!("missing required env var: {key}")))
}
