use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::case::{
    Case, CaseActivity, CaseActivityType, CaseAssignment, CaseNote, CaseStatus,
};

pub struct CaseRepo;

impl CaseRepo {
    pub async fn create(
        db: &PgPool,
        case_number: &str,
        investigation_type: &str,
        subject_plaintiff: Option<&str>,
        client_firm: Option<&str>,
        client_client: Option<&str>,
        client_contact: Option<&str>,
        investigator: Option<&str>,
        case_notes_area: Option<&str>,
        client_file_number: Option<&str>,
        additional_info: Option<&str>,
        expenses: Decimal,
        status: CaseStatus,
        date_assigned: Option<NaiveDate>,
        date_completed_paid: Option<NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AppResult<Case> {
        let case = sqlx::query_as::<_, Case>(
            r#"INSERT INTO cases (
                case_number, investigation_type, subject_plaintiff, client_firm,
                client_client, client_contact, investigator, case_notes_area,
                client_file_number, additional_info, expenses, status,
                date_assigned, date_completed_paid, created_by
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15
            ) RETURNING *"#,
        )
        .bind(case_number)
        .bind(investigation_type)
        .bind(subject_plaintiff)
        .bind(client_firm)
        .bind(client_client)
        .bind(client_contact)
        .bind(investigator)
        .bind(case_notes_area)
        .bind(client_file_number)
        .bind(additional_info)
        .bind(expenses)
        .bind(status)
        .bind(date_assigned)
        .bind(date_completed_paid)
        .bind(created_by)
        .fetch_one(db)
        .await
        .map_err(map_unique)?;
        Ok(case)
    }

    pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<Case>> {
        Ok(sqlx::query_as::<_, Case>("SELECT * FROM cases WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?)
    }

    pub async fn find_by_case_number(db: &PgPool, case_number: &str) -> AppResult<Option<Case>> {
        Ok(
            sqlx::query_as::<_, Case>("SELECT * FROM cases WHERE case_number = $1")
                .bind(case_number)
                .fetch_optional(db)
                .await?,
        )
    }

    pub async fn next_case_number(db: &PgPool, year_yy: &str) -> AppResult<String> {
        let pattern = format!("%-{year_yy}");
        let max: Option<i32> = sqlx::query_scalar(
            r#"SELECT MAX(CAST(SPLIT_PART(case_number, '-', 1) AS INTEGER))
               FROM cases
               WHERE case_number LIKE $1
                 AND SPLIT_PART(case_number, '-', 1) ~ '^[0-9]+$'"#,
        )
        .bind(&pattern)
        .fetch_one(db)
        .await?;
        Ok(format!("{}-{}", max.unwrap_or(0) + 1, year_yy))
    }

    pub async fn list(
        db: &PgPool,
        status: Option<CaseStatus>,
        investigator: Option<&str>,
        q: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Case>, i64)> {
        let search = q.map(|s| format!("%{}%", s.trim())).filter(|s| s.len() > 2);

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*)::bigint FROM cases
               WHERE ($1::case_status IS NULL OR status = $1)
                 AND ($2::text IS NULL OR investigator = $2)
                 AND (
                   $3::text IS NULL
                   OR case_number ILIKE $3
                   OR COALESCE(subject_plaintiff, '') ILIKE $3
                   OR COALESCE(client_firm, '') ILIKE $3
                   OR COALESCE(client_file_number, '') ILIKE $3
                   OR investigation_type ILIKE $3
                 )"#,
        )
        .bind(status)
        .bind(investigator)
        .bind(search.as_deref())
        .fetch_one(db)
        .await?;

        let items = sqlx::query_as::<_, Case>(
            r#"SELECT * FROM cases
               WHERE ($1::case_status IS NULL OR status = $1)
                 AND ($2::text IS NULL OR investigator = $2)
                 AND (
                   $3::text IS NULL
                   OR case_number ILIKE $3
                   OR COALESCE(subject_plaintiff, '') ILIKE $3
                   OR COALESCE(client_firm, '') ILIKE $3
                   OR COALESCE(client_file_number, '') ILIKE $3
                   OR investigation_type ILIKE $3
                 )
               ORDER BY COALESCE(date_assigned, created_at::date) DESC, created_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(status)
        .bind(investigator)
        .bind(search.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;

        Ok((items, total))
    }

    pub async fn update(db: &PgPool, case: &Case) -> AppResult<Case> {
        let updated = sqlx::query_as::<_, Case>(
            r#"UPDATE cases SET
                investigation_type = $2,
                subject_plaintiff = $3,
                client_firm = $4,
                client_client = $5,
                client_contact = $6,
                investigator = $7,
                case_notes_area = $8,
                client_file_number = $9,
                additional_info = $10,
                expenses = $11,
                status = $12,
                date_assigned = $13,
                date_completed_paid = $14,
                updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(case.id)
        .bind(&case.investigation_type)
        .bind(&case.subject_plaintiff)
        .bind(&case.client_firm)
        .bind(&case.client_client)
        .bind(&case.client_contact)
        .bind(&case.investigator)
        .bind(&case.case_notes_area)
        .bind(&case.client_file_number)
        .bind(&case.additional_info)
        .bind(case.expenses)
        .bind(case.status)
        .bind(case.date_assigned)
        .bind(case.date_completed_paid)
        .fetch_one(db)
        .await?;
        Ok(updated)
    }

    pub async fn delete(db: &PgPool, id: Uuid) -> AppResult<bool> {
        let res = sqlx::query("DELETE FROM cases WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn log_activity(
        db: &PgPool,
        case_id: Uuid,
        user_id: Option<Uuid>,
        activity_type: CaseActivityType,
        notes: Option<&str>,
    ) -> AppResult<CaseActivity> {
        let row = sqlx::query_as::<_, CaseActivity>(
            r#"INSERT INTO case_activities (case_id, user_id, activity_type, notes)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(case_id)
        .bind(user_id)
        .bind(activity_type)
        .bind(notes)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    pub async fn list_activities(db: &PgPool, case_id: Uuid) -> AppResult<Vec<CaseActivity>> {
        Ok(sqlx::query_as::<_, CaseActivity>(
            r#"SELECT * FROM case_activities
               WHERE case_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(case_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn add_note(
        db: &PgPool,
        case_id: Uuid,
        user_id: Option<Uuid>,
        note: &str,
        is_internal: bool,
    ) -> AppResult<CaseNote> {
        let row = sqlx::query_as::<_, CaseNote>(
            r#"INSERT INTO case_notes (case_id, user_id, note, is_internal)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(case_id)
        .bind(user_id)
        .bind(note)
        .bind(is_internal)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    pub async fn list_notes(db: &PgPool, case_id: Uuid) -> AppResult<Vec<CaseNote>> {
        Ok(sqlx::query_as::<_, CaseNote>(
            r#"SELECT * FROM case_notes
               WHERE case_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(case_id)
        .fetch_all(db)
        .await?)
    }

    pub async fn record_assignment(
        db: &PgPool,
        case_id: Uuid,
        assigned_to: &str,
        assigned_by: Option<Uuid>,
        reason: Option<&str>,
    ) -> AppResult<CaseAssignment> {
        let row = sqlx::query_as::<_, CaseAssignment>(
            r#"INSERT INTO case_assignments (case_id, assigned_to, assigned_by, reason)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(case_id)
        .bind(assigned_to)
        .bind(assigned_by)
        .bind(reason)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    pub async fn stats(db: &PgPool) -> AppResult<crate::dto::cases::CaseStats> {
        let row = sqlx::query_as::<_, StatusCounts>(
            r#"SELECT
                COUNT(*)::bigint AS total,
                COUNT(*) FILTER (WHERE status = 'not_started')::bigint AS not_started,
                COUNT(*) FILTER (WHERE status = 'contact_attempted')::bigint AS contact_attempted,
                COUNT(*) FILTER (WHERE status = 'client_contacted')::bigint AS client_contacted,
                COUNT(*) FILTER (WHERE status = 'inspection_scheduled')::bigint AS inspection_scheduled,
                COUNT(*) FILTER (WHERE status = 'scene_inspected')::bigint AS scene_inspected,
                COUNT(*) FILTER (WHERE status = 'report_sent')::bigint AS report_sent,
                COUNT(*) FILTER (WHERE status = 'completed')::bigint AS completed,
                COUNT(*) FILTER (WHERE status = 'cancelled')::bigint AS cancelled,
                COUNT(*) FILTER (WHERE status = 'stalled')::bigint AS stalled
               FROM cases"#,
        )
        .fetch_one(db)
        .await?;

        Ok(crate::dto::cases::CaseStats {
            total: row.total,
            not_started: row.not_started,
            contact_attempted: row.contact_attempted,
            client_contacted: row.client_contacted,
            inspection_scheduled: row.inspection_scheduled,
            scene_inspected: row.scene_inspected,
            report_sent: row.report_sent,
            completed: row.completed,
            cancelled: row.cancelled,
            stalled: row.stalled,
        })
    }

    pub async fn investigator_stats(
        db: &PgPool,
    ) -> AppResult<Vec<crate::dto::cases::InvestigatorCaseStats>> {
        let rows = sqlx::query_as::<_, InvestigatorCounts>(
            r#"SELECT
                investigator AS investigator,
                COUNT(*)::bigint AS total,
                COUNT(*) FILTER (WHERE status NOT IN ('completed', 'cancelled'))::bigint AS active,
                COUNT(*) FILTER (WHERE status = 'completed')::bigint AS completed
               FROM cases
               WHERE investigator IS NOT NULL AND investigator <> ''
               GROUP BY investigator
               ORDER BY total DESC"#,
        )
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| crate::dto::cases::InvestigatorCaseStats {
                investigator: r.investigator,
                total: r.total,
                active: r.active,
                completed: r.completed,
            })
            .collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StatusCounts {
    total: i64,
    not_started: i64,
    contact_attempted: i64,
    client_contacted: i64,
    inspection_scheduled: i64,
    scene_inspected: i64,
    report_sent: i64,
    completed: i64,
    cancelled: i64,
    stalled: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct InvestigatorCounts {
    investigator: String,
    total: i64,
    active: i64,
    completed: i64,
}

fn map_unique(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::Database(db) if db.constraint() == Some("cases_case_number_key") => {
            AppError::Conflict("case_number already exists".into())
        }
        _ => AppError::Database(err),
    }
}
