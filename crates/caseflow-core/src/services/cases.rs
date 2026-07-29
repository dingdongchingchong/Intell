use chrono::{Datelike, NaiveDate, Utc};
use sqlx::{Postgres, QueryBuilder, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    Case, CaseListQuery, CasePriority, CaseStage, CreateCaseRequest, DashboardStats,
    ImportCaseError, ImportCasesRequest, ImportCasesResponse, StageCount, UpdateCaseRequest,
    UpdateStageRequest,
};
use crate::services::audit;

pub async fn next_case_number(pool: &PgPool) -> AppResult<String> {
    let year = Utc::now().year() % 100;
    let prefix = format!("{year:02}-");
    let latest: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT case_number FROM cases
        WHERE case_number LIKE $1
        ORDER BY case_number DESC
        LIMIT 1
        "#,
    )
    .bind(format!("{prefix}%"))
    .fetch_optional(pool)
    .await?;

    let next = match latest {
        Some((num,)) => {
            let seq = num
                .split('-')
                .nth(1)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0)
                + 1;
            seq
        }
        None => 1,
    };
    Ok(format!("{prefix}{next:04}"))
}

pub async fn list_cases(pool: &PgPool, q: CaseListQuery) -> AppResult<(Vec<Case>, i64)> {
    let page = q.page.unwrap_or(0).max(0);
    let page_size = q.page_size.unwrap_or(50).clamp(1, 200);
    let offset = page * page_size;

    let mut count_qb: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*) FROM cases WHERE deleted_at IS NULL");
    let mut list_qb: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT * FROM cases WHERE deleted_at IS NULL");

    if let Some(ref search) = q.search {
        let s = format!("%{}%", search.trim());
        for qb in [&mut count_qb, &mut list_qb] {
            qb.push(" AND (case_number ILIKE ");
            qb.push_bind(s.clone());
            qb.push(" OR subject ILIKE ");
            qb.push_bind(s.clone());
            qb.push(" OR client ILIKE ");
            qb.push_bind(s.clone());
            qb.push(")");
        }
    }
    if let Some(ref stage) = q.stage {
        for qb in [&mut count_qb, &mut list_qb] {
            qb.push(" AND stage = ");
            qb.push_bind(stage.clone());
        }
    }
    if let Some(id) = q.investigator_id {
        for qb in [&mut count_qb, &mut list_qb] {
            qb.push(" AND investigator_id = ");
            qb.push_bind(id);
        }
    }
    if let Some(ref priority) = q.priority {
        for qb in [&mut count_qb, &mut list_qb] {
            qb.push(" AND priority = ");
            qb.push_bind(priority.clone());
        }
    }

    let total: (i64,) = count_qb.build_query_as().fetch_one(pool).await?;

    list_qb.push(" ORDER BY opened_date DESC, created_at DESC LIMIT ");
    list_qb.push_bind(page_size);
    list_qb.push(" OFFSET ");
    list_qb.push_bind(offset);

    let rows: Vec<Case> = list_qb.build_query_as().fetch_all(pool).await?;
    Ok((rows, total.0))
}

pub async fn get_case(pool: &PgPool, id: Uuid) -> AppResult<Case> {
    sqlx::query_as::<_, Case>(
        "SELECT * FROM cases WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("case not found".into()))
}

pub async fn create_case(
    pool: &PgPool,
    actor_id: Uuid,
    req: CreateCaseRequest,
) -> AppResult<Case> {
    if req.subject.trim().len() < 2 {
        return Err(AppError::Validation("subject required".into()));
    }
    if req.client.trim().is_empty() {
        return Err(AppError::Validation("client required".into()));
    }

    let case_number = match req.case_number.filter(|s| !s.trim().is_empty()) {
        Some(n) => n.trim().to_string(),
        None => next_case_number(pool).await?,
    };

    let stage = req
        .stage
        .as_deref()
        .and_then(CaseStage::parse)
        .unwrap_or(CaseStage::NotStarted);
    let priority = req
        .priority
        .as_deref()
        .and_then(CasePriority::parse)
        .unwrap_or(CasePriority::Normal);
    let opened = req.opened_date.unwrap_or_else(|| Utc::now().date_naive());
    let is_rush = req.is_rush.unwrap_or(priority == CasePriority::Rush || priority == CasePriority::Urgent);

    let case = sqlx::query_as::<_, Case>(
        r#"
        INSERT INTO cases (
            case_number, subject, investigation_type, client, clients_client,
            client_contact, client_file, investigator_id, assigned_to_id,
            stage, priority, is_rush, is_rework, is_death, opened_date,
            completed_date, case_notes, additional_info, case_status,
            created_by, assigned_at, assigned_by, stage_changed_at
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
            CASE WHEN $9 IS NOT NULL THEN NOW() ELSE NULL END,
            CASE WHEN $9 IS NOT NULL THEN $20 ELSE NULL END,
            NOW()
        )
        RETURNING *
        "#,
    )
    .bind(&case_number)
    .bind(req.subject.trim())
    .bind(req.investigation_type.trim())
    .bind(req.client.trim())
    .bind(req.clients_client)
    .bind(req.client_contact)
    .bind(req.client_file)
    .bind(req.investigator_id)
    .bind(req.assigned_to_id)
    .bind(stage.as_str())
    .bind(priority.as_str())
    .bind(is_rush)
    .bind(req.is_rework.unwrap_or(false))
    .bind(req.is_death.unwrap_or(false))
    .bind(opened)
    .bind(req.completed_date)
    .bind(req.case_notes)
    .bind(req.additional_info)
    .bind(req.case_status)
    .bind(actor_id)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => {
            AppError::Conflict("case number already exists".into())
        }
        other => AppError::from(other),
    })?;

    // Persist client name for dropdowns
    let _ = sqlx::query(
        "INSERT INTO clients (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
    )
    .bind(req.client.trim())
    .execute(pool)
    .await;

    audit::log(
        pool,
        Some(actor_id),
        "case_create",
        "case",
        Some(&case.id.to_string()),
        Some(&case.case_number),
    )
    .await?;

    Ok(case)
}

pub async fn update_case(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    req: UpdateCaseRequest,
) -> AppResult<Case> {
    let existing = get_case(pool, id).await?;

    let stage = req
        .stage
        .as_deref()
        .and_then(CaseStage::parse)
        .map(|s| s.as_str().to_string())
        .unwrap_or(existing.stage.clone());
    let priority = req
        .priority
        .as_deref()
        .and_then(CasePriority::parse)
        .map(|s| s.as_str().to_string())
        .unwrap_or(existing.priority.clone());

    let stage_changed = stage != existing.stage;

    let case = sqlx::query_as::<_, Case>(
        r#"
        UPDATE cases SET
            subject = COALESCE($2, subject),
            investigation_type = COALESCE($3, investigation_type),
            client = COALESCE($4, client),
            clients_client = COALESCE($5, clients_client),
            client_contact = COALESCE($6, client_contact),
            client_file = COALESCE($7, client_file),
            investigator_id = COALESCE($8, investigator_id),
            assigned_to_id = COALESCE($9, assigned_to_id),
            stage = $10,
            priority = $11,
            is_rush = COALESCE($12, is_rush),
            is_rework = COALESCE($13, is_rework),
            is_death = COALESCE($14, is_death),
            case_notes = COALESCE($15, case_notes),
            additional_info = COALESCE($16, additional_info),
            completed_date = COALESCE($17, completed_date),
            opened_date = COALESCE($18, opened_date),
            case_status = COALESCE($19, case_status),
            stage_changed_at = CASE WHEN $20 THEN NOW() ELSE stage_changed_at END,
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(req.subject.as_deref())
    .bind(req.investigation_type.as_deref())
    .bind(req.client.as_deref())
    .bind(req.clients_client.as_deref())
    .bind(req.client_contact.as_deref())
    .bind(req.client_file.as_deref())
    .bind(req.investigator_id)
    .bind(req.assigned_to_id)
    .bind(&stage)
    .bind(&priority)
    .bind(req.is_rush)
    .bind(req.is_rework)
    .bind(req.is_death)
    .bind(req.case_notes.as_deref())
    .bind(req.additional_info.as_deref())
    .bind(req.completed_date)
    .bind(req.opened_date)
    .bind(req.case_status.as_deref())
    .bind(stage_changed)
    .fetch_one(pool)
    .await?;

    audit::log(
        pool,
        Some(actor_id),
        "case_update",
        "case",
        Some(&id.to_string()),
        None,
    )
    .await?;

    Ok(case)
}

pub async fn update_stage(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    req: UpdateStageRequest,
) -> AppResult<Case> {
    let stage = CaseStage::parse(&req.stage)
        .ok_or_else(|| AppError::Validation("invalid stage".into()))?;
    let completed: Option<NaiveDate> = if matches!(stage, CaseStage::Completed) {
        Some(Utc::now().date_naive())
    } else {
        None
    };

    let case = sqlx::query_as::<_, Case>(
        r#"
        UPDATE cases SET
            stage = $2,
            completed_date = COALESCE($3, completed_date),
            stage_changed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(stage.as_str())
    .bind(completed)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("case not found".into()))?;

    if let Some(note) = req.note.filter(|n| !n.trim().is_empty()) {
        sqlx::query(
            r#"
            INSERT INTO case_notes (case_id, user_id, content, is_internal)
            VALUES ($1, $2, $3, true)
            "#,
        )
        .bind(id)
        .bind(actor_id)
        .bind(note)
        .execute(pool)
        .await?;
    }

    audit::log(
        pool,
        Some(actor_id),
        "stage_change",
        "case",
        Some(&id.to_string()),
        Some(stage.as_str()),
    )
    .await?;

    Ok(case)
}

pub async fn soft_delete(pool: &PgPool, actor_id: Uuid, id: Uuid) -> AppResult<()> {
    let res = sqlx::query(
        "UPDATE cases SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("case not found".into()));
    }
    audit::log(
        pool,
        Some(actor_id),
        "case_delete",
        "case",
        Some(&id.to_string()),
        None,
    )
    .await?;
    Ok(())
}

pub async fn dashboard_stats(pool: &PgPool) -> AppResult<DashboardStats> {
    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM cases WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;
    let completed: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cases WHERE deleted_at IS NULL AND stage = 'completed'",
    )
    .fetch_one(pool)
    .await?;
    let rush: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cases WHERE deleted_at IS NULL AND is_rush = true AND stage NOT IN ('completed','cancelled')",
    )
    .fetch_one(pool)
    .await?;
    let stalled: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM cases
        WHERE deleted_at IS NULL
          AND stage NOT IN ('completed','cancelled')
          AND COALESCE(stage_changed_at, created_at) < NOW() - INTERVAL '10 days'
        "#,
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT stage, COUNT(*)::bigint
        FROM cases WHERE deleted_at IS NULL
        GROUP BY stage
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(DashboardStats {
        total_cases: total.0,
        active_cases: total.0 - completed.0,
        completed_cases: completed.0,
        rush_cases: rush.0,
        stalled_cases: stalled.0,
        by_stage: rows
            .into_iter()
            .map(|(stage, count)| StageCount { stage, count })
            .collect(),
    })
}

pub async fn list_clients(pool: &PgPool) -> AppResult<Vec<String>> {
    let from_table: Vec<(String,)> = sqlx::query_as("SELECT name FROM clients ORDER BY name")
        .fetch_all(pool)
        .await?;
    let mut names: Vec<String> = from_table.into_iter().map(|r| r.0).collect();
    let from_cases: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT client FROM cases WHERE deleted_at IS NULL ORDER BY client",
    )
    .fetch_all(pool)
    .await?;
    for (c,) in from_cases {
        if !names.iter().any(|n| n == &c) {
            names.push(c);
        }
    }
    names.sort();
    Ok(names)
}

pub async fn import_cases(
    pool: &PgPool,
    actor_id: Uuid,
    req: ImportCasesRequest,
) -> AppResult<ImportCasesResponse> {
    if req.cases.is_empty() {
        return Err(AppError::Validation("no cases to import".into()));
    }
    if req.cases.len() > 5000 {
        return Err(AppError::Validation("import limited to 5000 rows".into()));
    }

    let update_existing = req.update_existing.unwrap_or(true);
    let mut created = 0i64;
    let mut updated = 0i64;
    let mut skipped = 0i64;
    let mut errors = Vec::new();

    for (idx, row) in req.cases.into_iter().enumerate() {
        let row_num = idx + 1;
        let case_number = row
            .case_number
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let existing_id: Option<Uuid> = if let Some(ref cn) = case_number {
            sqlx::query_scalar(
                "SELECT id FROM cases WHERE case_number = $1 AND deleted_at IS NULL",
            )
            .bind(cn)
            .fetch_optional(pool)
            .await?
        } else {
            None
        };

        if let Some(id) = existing_id {
            if !update_existing {
                skipped += 1;
                continue;
            }
            let update = UpdateCaseRequest {
                subject: Some(row.subject.clone()),
                investigation_type: Some(row.investigation_type.clone()),
                client: Some(row.client.clone()),
                clients_client: row.clients_client.clone(),
                client_contact: row.client_contact.clone(),
                client_file: row.client_file.clone(),
                investigator_id: row.investigator_id,
                assigned_to_id: row.assigned_to_id,
                stage: row.stage.clone(),
                priority: row.priority.clone(),
                is_rush: row.is_rush,
                is_rework: row.is_rework,
                is_death: row.is_death,
                opened_date: row.opened_date,
                completed_date: row.completed_date,
                case_notes: row.case_notes.clone(),
                additional_info: row.additional_info.clone(),
                case_status: row.case_status.clone(),
            };
            match update_case(pool, actor_id, id, update).await {
                Ok(_) => updated += 1,
                Err(e) => errors.push(ImportCaseError {
                    row: row_num,
                    case_number,
                    message: e.to_string(),
                }),
            }
        } else {
            match create_case(pool, actor_id, row).await {
                Ok(_) => created += 1,
                Err(e) => errors.push(ImportCaseError {
                    row: row_num,
                    case_number,
                    message: e.to_string(),
                }),
            }
        }
    }

    audit::log(
        pool,
        Some(actor_id),
        "case_import",
        "case",
        None,
        Some(&format!(
            "created={created} updated={updated} skipped={skipped} errors={}",
            errors.len()
        )),
    )
    .await?;

    Ok(ImportCasesResponse {
        created,
        updated,
        skipped,
        errors,
    })
}

