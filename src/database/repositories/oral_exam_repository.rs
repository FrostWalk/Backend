use crate::models::oral_exam_completion::OralExamCompletion;
use crate::models::oral_exam_note::OralExamNote;
use chrono::{DateTime, Utc};
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

// ── Notes ──────────────────────────────────────────────────────────────────

pub(crate) async fn get_note(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<Option<DbState<OralExamNote>>> {
    let mut rows = OralExamNote::where_col(|n| n.student_id.equal(student_id))
        .where_col(|n| n.project_id.equal(project_id))
        .limit(1)
        .run(db)
        .await?;
    Ok(rows.pop())
}

pub(crate) async fn get_notes_for_project(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<OralExamNote>>> {
    OralExamNote::where_col(|n| n.project_id.equal(project_id))
        .run(db)
        .await
}

pub(crate) async fn upsert_note(
    db: &PostgresClient, student_id: i32, project_id: i32, note_text: String,
    updated_by_admin_id: i32, now: DateTime<Utc>,
) -> welds::errors::Result<DbState<OralExamNote>> {
    if let Some(mut existing) = get_note(db, student_id, project_id).await? {
        existing.as_mut().note_text = note_text;
        existing.as_mut().updated_at = now;
        existing.as_mut().updated_by_admin_id = Some(updated_by_admin_id);
        existing.save(db).await?;
        Ok(existing)
    } else {
        let note = OralExamNote {
            note_id: 0,
            student_id,
            project_id,
            note_text,
            updated_at: now,
            updated_by_admin_id: Some(updated_by_admin_id),
        };
        let mut state = DbState::new_uncreated(note);
        state.save(db).await?;
        Ok(state)
    }
}

pub(crate) async fn delete_note(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<bool> {
    let rows = OralExamNote::where_col(|n| n.student_id.equal(student_id))
        .where_col(|n| n.project_id.equal(project_id))
        .run(db)
        .await?;
    if rows.is_empty() {
        return Ok(false);
    }
    OralExamNote::where_col(|n| n.student_id.equal(student_id))
        .where_col(|n| n.project_id.equal(project_id))
        .delete(db)
        .await?;
    Ok(true)
}

// ── Completions ────────────────────────────────────────────────────────────

pub(crate) async fn get_completion(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<Option<DbState<OralExamCompletion>>> {
    let mut rows = OralExamCompletion::where_col(|c| c.student_id.equal(student_id))
        .where_col(|c| c.project_id.equal(project_id))
        .limit(1)
        .run(db)
        .await?;
    Ok(rows.pop())
}

pub(crate) async fn get_completions_for_project(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<OralExamCompletion>>> {
    OralExamCompletion::where_col(|c| c.project_id.equal(project_id))
        .run(db)
        .await
}

/// Mark student as completed. If already completed, updates timestamp and admin.
pub(crate) async fn mark_completed(
    db: &PostgresClient, student_id: i32, project_id: i32, completed_by_admin_id: i32,
    now: DateTime<Utc>,
) -> welds::errors::Result<DbState<OralExamCompletion>> {
    if let Some(mut existing) = get_completion(db, student_id, project_id).await? {
        existing.as_mut().completed_at = now;
        existing.as_mut().completed_by_admin_id = Some(completed_by_admin_id);
        existing.save(db).await?;
        Ok(existing)
    } else {
        let completion = OralExamCompletion {
            completion_id: 0,
            student_id,
            project_id,
            completed_at: now,
            completed_by_admin_id: Some(completed_by_admin_id),
        };
        let mut state = DbState::new_uncreated(completion);
        state.save(db).await?;
        Ok(state)
    }
}

pub(crate) async fn mark_incomplete(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<bool> {
    let rows = OralExamCompletion::where_col(|c| c.student_id.equal(student_id))
        .where_col(|c| c.project_id.equal(project_id))
        .run(db)
        .await?;
    if rows.is_empty() {
        return Ok(false);
    }
    OralExamCompletion::where_col(|c| c.student_id.equal(student_id))
        .where_col(|c| c.project_id.equal(project_id))
        .delete(db)
        .await?;
    Ok(true)
}
