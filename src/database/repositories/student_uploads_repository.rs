use crate::models::student::Student;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_selection::StudentDeliverableSelection;
use crate::models::student_upload::StudentUpload;
use chrono::{DateTime, Utc};
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn get_by_selection_id(
    db: &PostgresClient, student_deliverable_selection_id: i32,
) -> welds::errors::Result<Option<DbState<StudentUpload>>> {
    let mut rows = StudentUpload::where_col(|upload| {
        upload
            .student_deliverable_selection_id
            .equal(student_deliverable_selection_id)
    })
    .limit(1)
    .run(db)
    .await?;
    Ok(rows.pop())
}

pub(crate) async fn upsert(
    db: &PostgresClient, student_deliverable_selection_id: i32, path: String, now: DateTime<Utc>,
) -> welds::errors::Result<DbState<StudentUpload>> {
    if let Some(mut existing) = get_by_selection_id(db, student_deliverable_selection_id).await? {
        let current_count = existing.as_ref().upload_count;
        existing.as_mut().path = path;
        existing.as_mut().timestamp = now;
        existing.as_mut().upload_count = current_count + 1;
        existing.save(db).await?;
        Ok(existing)
    } else {
        let upload = StudentUpload {
            upload_id: 0,
            student_deliverable_selection_id,
            path,
            upload_count: 1,
            timestamp: now,
        };
        let mut state = DbState::new_uncreated(upload);
        state.save(db).await?;
        Ok(state)
    }
}

pub(crate) async fn get_all_by_project(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<(StudentUpload, Student)>> {
    let deliverables =
        StudentDeliverable::where_col(|deliverable| deliverable.project_id.equal(project_id))
            .run(db)
            .await?;
    let deliverable_ids: Vec<i32> = deliverables
        .into_iter()
        .map(|state| state.as_ref().student_deliverable_id)
        .collect();

    if deliverable_ids.is_empty() {
        return Ok(Vec::new());
    }

    let selections = StudentDeliverableSelection::all().run(db).await?;
    let project_selections: Vec<(i32, i32)> = selections
        .into_iter()
        .filter_map(|selection| {
            let selection_ref = selection.as_ref();
            if deliverable_ids.contains(&selection_ref.student_deliverable_id) {
                Some((
                    selection_ref.student_deliverable_selection_id,
                    selection_ref.student_id,
                ))
            } else {
                None
            }
        })
        .collect();

    if project_selections.is_empty() {
        return Ok(Vec::new());
    }

    let uploads = StudentUpload::all().run(db).await?;
    let mut result = Vec::new();
    for upload_state in uploads {
        let upload = upload_state.as_ref();
        let selection_match = project_selections
            .iter()
            .find(|(selection_id, _)| *selection_id == upload.student_deliverable_selection_id);
        let Some((_, student_id)) = selection_match else {
            continue;
        };

        let mut students = Student::where_col(|student| student.student_id.equal(*student_id))
            .limit(1)
            .run(db)
            .await?;

        if let Some(student_state) = students.pop() {
            result.push((upload.clone(), DbState::into_inner(student_state)));
        }
    }

    Ok(result)
}
