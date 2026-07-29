use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::models::case::{Case, CaseActivity, CaseNote, CaseStatus};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCaseRequest {
    /// Optional; auto-generated as `{n}-{yy}` when omitted.
    pub case_number: Option<String>,
    #[validate(length(min = 1, max = 120))]
    pub investigation_type: String,
    pub subject_plaintiff: Option<String>,
    pub client_firm: Option<String>,
    pub client_client: Option<String>,
    pub client_contact: Option<String>,
    pub investigator: Option<String>,
    pub case_notes_area: Option<String>,
    pub client_file_number: Option<String>,
    pub additional_info: Option<String>,
    #[schema(value_type = Option<String>)]
    pub expenses: Option<Decimal>,
    pub status: Option<CaseStatus>,
    pub date_assigned: Option<NaiveDate>,
    pub date_completed_paid: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCaseRequest {
    pub investigation_type: Option<String>,
    pub subject_plaintiff: Option<String>,
    pub client_firm: Option<String>,
    pub client_client: Option<String>,
    pub client_contact: Option<String>,
    pub investigator: Option<String>,
    pub case_notes_area: Option<String>,
    pub client_file_number: Option<String>,
    pub additional_info: Option<String>,
    #[schema(value_type = Option<String>)]
    pub expenses: Option<Decimal>,
    pub status: Option<CaseStatus>,
    pub date_assigned: Option<NaiveDate>,
    pub date_completed_paid: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignCaseRequest {
    #[validate(length(min = 1, max = 50))]
    pub investigator: String,
    pub reason: Option<String>,
    /// When true (default), sets status to `contact_attempted` if still `not_started`.
    pub bump_status: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCaseNoteRequest {
    #[validate(length(min = 1))]
    pub note: String,
    pub is_internal: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CaseListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per")]
    pub per_page: u32,
    pub status: Option<CaseStatus>,
    pub investigator: Option<String>,
    pub q: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_per() -> u32 {
    50
}

impl CaseListQuery {
    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, 200) as i64
    }

    pub fn offset(&self) -> i64 {
        ((self.page.max(1) - 1) * self.per_page.clamp(1, 200)) as i64
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CaseStats {
    pub total: i64,
    pub not_started: i64,
    pub contact_attempted: i64,
    pub client_contacted: i64,
    pub inspection_scheduled: i64,
    pub scene_inspected: i64,
    pub report_sent: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub stalled: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvestigatorCaseStats {
    pub investigator: String,
    pub total: i64,
    pub active: i64,
    pub completed: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CaseDetail {
    pub case: Case,
    pub notes: Vec<CaseNote>,
    pub activities: Vec<CaseActivity>,
}
