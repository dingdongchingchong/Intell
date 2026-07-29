use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::cases::{
    AssignCaseRequest, CaseDetail, CaseListQuery, CaseStats, CreateCaseNoteRequest,
    CreateCaseRequest, InvestigatorCaseStats, UpdateCaseRequest,
};
use crate::dto::common::Paginated;
use crate::error::{AppError, AppResult};
use crate::models::case::{Case, CaseActivity, CaseActivityType, CaseNote, CaseStatus};
use crate::models::user::User;
use crate::repositories::CaseRepo;
use crate::state::AppState;

pub struct CaseService;

impl CaseService {
    pub async fn list(state: &AppState, q: CaseListQuery) -> AppResult<Paginated<Case>> {
        let (items, total) = CaseRepo::list(
            &state.db,
            q.status,
            q.investigator.as_deref(),
            q.q.as_deref(),
            q.limit(),
            q.offset(),
        )
        .await?;
        Ok(Paginated {
            items,
            page: q.page.max(1),
            per_page: q.per_page.clamp(1, 200),
            total,
        })
    }

    pub async fn get(state: &AppState, id_or_number: &str) -> AppResult<CaseDetail> {
        let case = Self::resolve(state, id_or_number).await?;
        let notes = CaseRepo::list_notes(&state.db, case.id).await?;
        let activities = CaseRepo::list_activities(&state.db, case.id).await?;
        Ok(CaseDetail {
            case,
            notes,
            activities,
        })
    }

    pub async fn create(
        state: &AppState,
        actor: &User,
        req: CreateCaseRequest,
    ) -> AppResult<Case> {
        if !actor.role.can_edit() {
            return Err(AppError::Forbidden("insufficient role to create cases".into()));
        }

        let case_number = match req.case_number {
            Some(n) => {
                let n = n.trim().to_string();
                if n.is_empty() {
                    return Err(AppError::Validation("case_number cannot be empty".into()));
                }
                if CaseRepo::find_by_case_number(&state.db, &n).await?.is_some() {
                    return Err(AppError::Conflict("case_number already exists".into()));
                }
                n
            }
            None => {
                let yy = Utc::now().format("%y").to_string();
                CaseRepo::next_case_number(&state.db, &yy).await?
            }
        };

        let status = req.status.unwrap_or(CaseStatus::NotStarted);
        let expenses = req.expenses.unwrap_or(Decimal::ZERO);

        let case = CaseRepo::create(
            &state.db,
            &case_number,
            req.investigation_type.trim(),
            req.subject_plaintiff.as_deref(),
            req.client_firm.as_deref(),
            req.client_client.as_deref(),
            req.client_contact.as_deref(),
            req.investigator.as_deref(),
            req.case_notes_area.as_deref(),
            req.client_file_number.as_deref(),
            req.additional_info.as_deref(),
            expenses,
            status,
            req.date_assigned,
            req.date_completed_paid,
            Some(actor.id),
        )
        .await?;

        CaseRepo::log_activity(
            &state.db,
            case.id,
            Some(actor.id),
            CaseActivityType::Created,
            Some("Case created"),
        )
        .await?;

        if let Some(inv) = case.investigator.as_deref() {
            CaseRepo::record_assignment(
                &state.db,
                case.id,
                inv,
                Some(actor.id),
                Some("set on create"),
            )
            .await?;
            CaseRepo::log_activity(
                &state.db,
                case.id,
                Some(actor.id),
                CaseActivityType::Assigned,
                Some(&format!("Assigned to {inv}")),
            )
            .await?;
        }

        Ok(case)
    }

    pub async fn update(
        state: &AppState,
        actor: &User,
        id_or_number: &str,
        req: UpdateCaseRequest,
    ) -> AppResult<Case> {
        if !actor.role.can_edit() {
            return Err(AppError::Forbidden("insufficient role to update cases".into()));
        }

        let mut case = Self::resolve(state, id_or_number).await?;
        let old_status = case.status;
        let old_investigator = case.investigator.clone();

        if let Some(v) = req.investigation_type {
            let v = v.trim().to_string();
            if v.is_empty() {
                return Err(AppError::Validation("investigation_type cannot be empty".into()));
            }
            case.investigation_type = v;
        }
        if let Some(v) = req.subject_plaintiff {
            case.subject_plaintiff = empty_to_none(v);
        }
        if let Some(v) = req.client_firm {
            case.client_firm = empty_to_none(v);
        }
        if let Some(v) = req.client_client {
            case.client_client = empty_to_none(v);
        }
        if let Some(v) = req.client_contact {
            case.client_contact = empty_to_none(v);
        }
        if let Some(v) = req.investigator {
            case.investigator = empty_to_none(v);
        }
        if let Some(v) = req.case_notes_area {
            case.case_notes_area = empty_to_none(v);
        }
        if let Some(v) = req.client_file_number {
            case.client_file_number = empty_to_none(v);
        }
        if let Some(v) = req.additional_info {
            case.additional_info = empty_to_none(v);
        }
        if let Some(v) = req.expenses {
            case.expenses = v;
        }
        if let Some(v) = req.status {
            case.status = v;
        }
        if let Some(v) = req.date_assigned {
            case.date_assigned = Some(v);
        }
        if let Some(v) = req.date_completed_paid {
            case.date_completed_paid = Some(v);
        }

        let updated = CaseRepo::update(&state.db, &case).await?;

        if old_status != updated.status {
            CaseRepo::log_activity(
                &state.db,
                updated.id,
                Some(actor.id),
                CaseActivityType::StatusChange,
                Some(&format!("Status changed from {old_status} to {}", updated.status)),
            )
            .await?;
        } else {
            CaseRepo::log_activity(
                &state.db,
                updated.id,
                Some(actor.id),
                CaseActivityType::Updated,
                Some("Case updated"),
            )
            .await?;
        }

        if updated.investigator != old_investigator {
            if let Some(inv) = updated.investigator.as_deref() {
                CaseRepo::record_assignment(
                    &state.db,
                    updated.id,
                    inv,
                    Some(actor.id),
                    Some("updated via case edit"),
                )
                .await?;
                CaseRepo::log_activity(
                    &state.db,
                    updated.id,
                    Some(actor.id),
                    CaseActivityType::Assigned,
                    Some(&format!("Assigned to {inv}")),
                )
                .await?;
            }
        }

        Ok(updated)
    }

    pub async fn delete(state: &AppState, actor: &User, id_or_number: &str) -> AppResult<()> {
        if !actor.role.can_moderate() {
            return Err(AppError::Forbidden("editor or admin required to delete cases".into()));
        }
        let case = Self::resolve(state, id_or_number).await?;
        if !CaseRepo::delete(&state.db, case.id).await? {
            return Err(AppError::NotFound("case not found".into()));
        }
        Ok(())
    }

    pub async fn assign(
        state: &AppState,
        actor: &User,
        id_or_number: &str,
        req: AssignCaseRequest,
    ) -> AppResult<Case> {
        if !actor.role.can_edit() {
            return Err(AppError::Forbidden("insufficient role to assign cases".into()));
        }

        let mut case = Self::resolve(state, id_or_number).await?;
        let investigator = req.investigator.trim().to_string();
        if investigator.is_empty() {
            return Err(AppError::Validation("investigator cannot be empty".into()));
        }

        case.investigator = Some(investigator.clone());
        if case.date_assigned.is_none() {
            case.date_assigned = Some(Utc::now().date_naive());
        }
        let bump = req.bump_status.unwrap_or(true);
        if bump && case.status == CaseStatus::NotStarted {
            case.status = CaseStatus::ContactAttempted;
        }

        let updated = CaseRepo::update(&state.db, &case).await?;

        CaseRepo::record_assignment(
            &state.db,
            updated.id,
            &investigator,
            Some(actor.id),
            req.reason.as_deref(),
        )
        .await?;

        CaseRepo::log_activity(
            &state.db,
            updated.id,
            Some(actor.id),
            CaseActivityType::Assigned,
            Some(&format!("Assigned to {investigator}")),
        )
        .await?;

        Ok(updated)
    }

    pub async fn add_note(
        state: &AppState,
        actor: &User,
        id_or_number: &str,
        req: CreateCaseNoteRequest,
    ) -> AppResult<CaseNote> {
        if !actor.role.can_edit() {
            return Err(AppError::Forbidden("insufficient role to add notes".into()));
        }
        let case = Self::resolve(state, id_or_number).await?;
        let note = CaseRepo::add_note(
            &state.db,
            case.id,
            Some(actor.id),
            req.note.trim(),
            req.is_internal.unwrap_or(false),
        )
        .await?;
        CaseRepo::log_activity(
            &state.db,
            case.id,
            Some(actor.id),
            CaseActivityType::NoteAdded,
            Some("Note added"),
        )
        .await?;
        Ok(note)
    }

    pub async fn list_notes(state: &AppState, id_or_number: &str) -> AppResult<Vec<CaseNote>> {
        let case = Self::resolve(state, id_or_number).await?;
        CaseRepo::list_notes(&state.db, case.id).await
    }

    pub async fn list_activities(
        state: &AppState,
        id_or_number: &str,
    ) -> AppResult<Vec<CaseActivity>> {
        let case = Self::resolve(state, id_or_number).await?;
        CaseRepo::list_activities(&state.db, case.id).await
    }

    pub async fn stats(state: &AppState) -> AppResult<CaseStats> {
        CaseRepo::stats(&state.db).await
    }

    pub async fn investigator_stats(state: &AppState) -> AppResult<Vec<InvestigatorCaseStats>> {
        CaseRepo::investigator_stats(&state.db).await
    }

    async fn resolve(state: &AppState, id_or_number: &str) -> AppResult<Case> {
        if let Ok(id) = Uuid::parse_str(id_or_number) {
            if let Some(case) = CaseRepo::find_by_id(&state.db, id).await? {
                return Ok(case);
            }
        }
        CaseRepo::find_by_case_number(&state.db, id_or_number)
            .await?
            .ok_or_else(|| AppError::NotFound("case not found".into()))
    }
}

fn empty_to_none(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
