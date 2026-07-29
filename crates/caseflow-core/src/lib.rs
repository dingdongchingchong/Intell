//! CaseFlow core — domain models, auth, and SQLx data access.
//!
//! Note: Prax ORM was requested but remains early WIP. This crate uses
//! production-ready SQLx with typed models for Neon/Supabase Postgres.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod services;

pub use config::Settings;
pub use error::{AppError, AppResult};
pub use models::*;
