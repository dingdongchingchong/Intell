use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Manager,
    Investigator,
    Viewer,
}

impl UserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::Investigator => "investigator",
            Self::Viewer => "viewer",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "manager" => Some(Self::Manager),
            "investigator" => Some(Self::Investigator),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CaseStage {
    NotStarted,
    ContactAttempted,
    ClientContacted,
    InspectionScheduled,
    SceneInspected,
    ReportSent,
    Completed,
    Cancelled,
}

impl CaseStage {
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
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "not_started" => Some(Self::NotStarted),
            "contact_attempted" => Some(Self::ContactAttempted),
            "client_contacted" => Some(Self::ClientContacted),
            "inspection_scheduled" => Some(Self::InspectionScheduled),
            "scene_inspected" => Some(Self::SceneInspected),
            "report_sent" => Some(Self::ReportSent),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn all() -> &'static [CaseStage] {
        &[
            Self::NotStarted,
            Self::ContactAttempted,
            Self::ClientContacted,
            Self::InspectionScheduled,
            Self::SceneInspected,
            Self::ReportSent,
            Self::Completed,
            Self::Cancelled,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CasePriority {
    Normal,
    Rush,
    Urgent,
}

impl CasePriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Rush => "rush",
            Self::Urgent => "urgent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "rush" => Some(Self::Rush),
            "urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub name: String,
    pub avatar: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub role: String,
    pub name: String,
    pub avatar: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            username: u.username,
            role: u.role,
            name: u.name,
            avatar: u.avatar,
            is_active: u.is_active,
            last_login: u.last_login,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Case {
    pub id: Uuid,
    pub case_number: String,
    pub subject: String,
    pub investigation_type: String,
    pub client: String,
    pub clients_client: Option<String>,
    pub client_contact: Option<String>,
    pub client_file: Option<String>,
    pub investigator_id: Option<Uuid>,
    pub assigned_to_id: Option<Uuid>,
    pub stage: String,
    pub priority: String,
    pub is_rush: bool,
    pub is_rework: bool,
    pub is_death: bool,
    pub opened_date: NaiveDate,
    pub completed_date: Option<NaiveDate>,
    pub case_notes: Option<String>,
    pub additional_info: Option<String>,
    pub case_status: Option<String>,
    pub created_by: Uuid,
    pub assigned_at: Option<DateTime<Utc>>,
    pub assigned_by: Option<Uuid>,
    pub stage_changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CaseNote {
    pub id: Uuid,
    pub case_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_cases: i64,
    pub active_cases: i64,
    pub completed_cases: i64,
    pub rush_cases: i64,
    pub stalled_cases: i64,
    pub by_stage: Vec<StageCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCount {
    pub stage: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserPublic,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseRequest {
    pub case_number: Option<String>,
    pub subject: String,
    pub investigation_type: String,
    pub client: String,
    pub clients_client: Option<String>,
    pub client_contact: Option<String>,
    pub client_file: Option<String>,
    pub investigator_id: Option<Uuid>,
    pub assigned_to_id: Option<Uuid>,
    pub stage: Option<String>,
    pub priority: Option<String>,
    pub is_rush: Option<bool>,
    pub is_rework: Option<bool>,
    pub is_death: Option<bool>,
    pub opened_date: Option<NaiveDate>,
    pub case_notes: Option<String>,
    pub additional_info: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCaseRequest {
    pub subject: Option<String>,
    pub investigation_type: Option<String>,
    pub client: Option<String>,
    pub clients_client: Option<String>,
    pub client_contact: Option<String>,
    pub client_file: Option<String>,
    pub investigator_id: Option<Uuid>,
    pub assigned_to_id: Option<Uuid>,
    pub stage: Option<String>,
    pub priority: Option<String>,
    pub is_rush: Option<bool>,
    pub is_rework: Option<bool>,
    pub is_death: Option<bool>,
    pub case_notes: Option<String>,
    pub additional_info: Option<String>,
    pub completed_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStageRequest {
    pub stage: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct CaseListQuery {
    pub search: Option<String>,
    pub stage: Option<String>,
    pub investigator_id: Option<Uuid>,
    pub priority: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
