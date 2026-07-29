use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "case_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    NotStarted,
    ContactAttempted,
    ClientContacted,
    InspectionScheduled,
    SceneInspected,
    ReportSent,
    Completed,
    Cancelled,
    Stalled,
}

impl CaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::ContactAttempted => "contact_attempted",
            Self::ClientContacted => "client_contacted",
            Self::InspectionScheduled => "inspection_scheduled",
            Self::SceneInspected => "scene_inspected",
            Self::ReportSent => "report_sent",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Stalled => "stalled",
        }
    }
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "case_activity_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CaseActivityType {
    Created,
    Assigned,
    StatusChange,
    NoteAdded,
    SceneVisit,
    ClientContact,
    ReportSent,
    FilemailSent,
    ExpenseAdded,
    Completed,
    Updated,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Case {
    pub id: Uuid,
    pub case_number: String,
    pub investigation_type: String,
    pub subject_plaintiff: Option<String>,
    pub client_firm: Option<String>,
    pub client_client: Option<String>,
    pub client_contact: Option<String>,
    pub investigator: Option<String>,
    pub case_notes_area: Option<String>,
    pub client_file_number: Option<String>,
    pub additional_info: Option<String>,
    #[schema(value_type = String)]
    pub expenses: Decimal,
    pub status: CaseStatus,
    pub date_assigned: Option<NaiveDate>,
    pub date_completed_paid: Option<NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct CaseActivity {
    pub id: Uuid,
    pub case_id: Uuid,
    pub user_id: Option<Uuid>,
    pub activity_type: CaseActivityType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct CaseNote {
    pub id: Uuid,
    pub case_id: Uuid,
    pub user_id: Option<Uuid>,
    pub note: String,
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct CaseAssignment {
    pub id: Uuid,
    pub case_id: Uuid,
    pub assigned_to: String,
    pub assigned_by: Option<Uuid>,
    pub reason: Option<String>,
    pub assigned_at: DateTime<Utc>,
}
