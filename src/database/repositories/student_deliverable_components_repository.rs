use crate::models::student_deliverable_component::StudentDeliverableComponent;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all student deliverable components
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<StudentDeliverableComponent>>> {
    StudentDeliverableComponent::all().run(db).await
}

/// Get a student deliverable component by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<Option<DbState<StudentDeliverableComponent>>> {
    let mut rows = StudentDeliverableComponent::where_col(|sdc| {
        sdc.student_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await?;

    Ok(rows.pop())
}

/// Get all student deliverable components for a specific project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverableComponent>>> {
    StudentDeliverableComponent::where_col(|sdc| sdc.project_id.equal(project_id))
        .run(db)
        .await
}

/// Check if a student component with the same name exists in a project (excluding a specific ID)
pub(crate) async fn check_name_exists_excluding(
    db: &PostgresClient, project_id: i32, name: &str, excluding_id: i32,
) -> welds::errors::Result<bool> {
    let rows = StudentDeliverableComponent::where_col(|sdc| sdc.project_id.equal(project_id))
        .where_col(|sdc| sdc.name.equal(name))
        .where_col(|sdc| sdc.student_deliverable_component_id.not_equal(excluding_id))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Check if a student component with the same name exists in a project
pub(crate) async fn check_name_exists(
    db: &PostgresClient, project_id: i32, name: &str,
) -> welds::errors::Result<bool> {
    let rows = StudentDeliverableComponent::where_col(|sdc| sdc.project_id.equal(project_id))
        .where_col(|sdc| sdc.name.equal(name))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Delete a student deliverable component by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<()> {
    StudentDeliverableComponent::where_col(|sdc| {
        sdc.student_deliverable_component_id.equal(component_id)
    })
    .delete(db)
    .await?;
    Ok(())
}

/// Create a new student deliverable component
pub(crate) async fn create(
    db: &PostgresClient, student_deliverable_component: StudentDeliverableComponent,
) -> welds::errors::Result<DbState<StudentDeliverableComponent>> {
    let mut state = DbState::new_uncreated(student_deliverable_component);
    state.save(db).await?;
    Ok(state)
}

/// Update a student deliverable component
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<StudentDeliverableComponent>,
) -> welds::errors::Result<DbState<StudentDeliverableComponent>> {
    state.save(db).await?;
    Ok(state)
}
