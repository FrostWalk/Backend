use crate::models::student_deliverable::StudentDeliverable;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all student deliverables
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<StudentDeliverable>>> {
    StudentDeliverable::all().run(db).await
}

/// Get a student deliverable by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, student_deliverable_id: i32,
) -> welds::errors::Result<Option<DbState<StudentDeliverable>>> {
    let mut rows =
        StudentDeliverable::where_col(|sd| sd.student_deliverable_id.equal(student_deliverable_id))
            .run(db)
            .await?;

    Ok(rows.pop())
}

/// Get all student deliverables for a specific project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverable>>> {
    StudentDeliverable::where_col(|sd| sd.project_id.equal(project_id))
        .run(db)
        .await
}

/// Check if a student deliverable with the same name exists in a project (excluding a specific ID)
pub(crate) async fn check_name_exists_excluding(
    db: &PostgresClient, project_id: i32, name: &str, excluding_id: i32,
) -> welds::errors::Result<bool> {
    let rows = StudentDeliverable::where_col(|sd| sd.project_id.equal(project_id))
        .where_col(|sd| sd.name.equal(name))
        .where_col(|sd| sd.student_deliverable_id.not_equal(excluding_id))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Check if a student deliverable with the same name exists in a project
pub(crate) async fn check_name_exists(
    db: &PostgresClient, project_id: i32, name: &str,
) -> welds::errors::Result<bool> {
    let rows = StudentDeliverable::where_col(|sd| sd.project_id.equal(project_id))
        .where_col(|sd| sd.name.equal(name))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Delete a student deliverable by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, student_deliverable_id: i32,
) -> welds::errors::Result<()> {
    StudentDeliverable::where_col(|sd| sd.student_deliverable_id.equal(student_deliverable_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Create a new student deliverable
pub(crate) async fn create(
    db: &PostgresClient, student_deliverable: StudentDeliverable,
) -> welds::errors::Result<DbState<StudentDeliverable>> {
    let mut state = DbState::new_uncreated(student_deliverable);
    state.save(db).await?;
    Ok(state)
}

/// Update a student deliverable by ID
pub(crate) async fn update_by_id(
    db: &PostgresClient, student_deliverable_id: i32, name: &str,
) -> welds::errors::Result<()> {
    StudentDeliverable::where_col(|sd| sd.student_deliverable_id.equal(student_deliverable_id))
        .set(|sd| sd.name, name)
        .run(db)
        .await?;
    Ok(())
}
