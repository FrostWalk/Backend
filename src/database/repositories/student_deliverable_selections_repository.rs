use crate::models::student_deliverable_selection::StudentDeliverableSelection;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get a student deliverable selection by student ID and project ID
pub(crate) async fn get_by_student_and_project(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<Option<DbState<StudentDeliverableSelection>>> {
    // First, get all student deliverables for the project
    let student_deliverables =
        crate::models::student_deliverable::StudentDeliverable::where_col(|sd| {
            sd.project_id.equal(project_id)
        })
        .run(db)
        .await?;

    let deliverable_ids: Vec<i32> = student_deliverables
        .into_iter()
        .map(|state| DbState::into_inner(state).student_deliverable_id)
        .collect();

    // Get all selections for this student
    let selections = StudentDeliverableSelection::where_col(|sds| sds.student_id.equal(student_id))
        .run(db)
        .await?;

    // Find the selection that matches one of the deliverable IDs for this project
    for selection_state in selections {
        let selection_id = selection_state.as_ref().student_deliverable_id;
        if deliverable_ids.contains(&selection_id) {
            return Ok(Some(selection_state));
        }
    }

    Ok(None)
}

/// Check if a student has already selected a deliverable for a project
pub(crate) async fn has_selection_for_project(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<bool> {
    let selection = get_by_student_and_project(db, student_id, project_id).await?;
    Ok(selection.is_some())
}

/// Delete a student's deliverable selection for a specific project
pub(crate) async fn delete_by_student_and_project(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<()> {
    if let Some(selection_state) = get_by_student_and_project(db, student_id, project_id).await? {
        let selection = DbState::into_inner(selection_state);
        StudentDeliverableSelection::where_col(|sds| {
            sds.student_deliverable_selection_id
                .equal(selection.student_deliverable_selection_id)
        })
        .delete(db)
        .await?;
    }
    Ok(())
}

/// Get all student deliverable selections for a project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverableSelection>>> {
    // First, get all student deliverables for the project
    let student_deliverables =
        crate::models::student_deliverable::StudentDeliverable::where_col(|sd| {
            sd.project_id.equal(project_id)
        })
        .run(db)
        .await?;

    let deliverable_ids: Vec<i32> = student_deliverables
        .into_iter()
        .map(|state| DbState::into_inner(state).student_deliverable_id)
        .collect();

    // Get all selections that match these deliverable IDs
    let all_selections = StudentDeliverableSelection::all().run(db).await?;

    let mut result = Vec::new();
    for selection_state in all_selections {
        let selection_id = selection_state.as_ref().student_deliverable_id;
        if deliverable_ids.contains(&selection_id) {
            result.push(selection_state);
        }
    }

    Ok(result)
}
