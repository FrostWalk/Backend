use crate::models::coordinator_project::CoordinatorProject;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Create a coordinator-project assignment
pub(crate) async fn create(
    db: &PostgresClient, admin_id: i32, project_id: i32,
) -> welds::errors::Result<DbState<CoordinatorProject>> {
    let mut coordinator_project = DbState::new_uncreated(CoordinatorProject {
        coordinator_project_id: 0,
        admin_id,
        project_id,
        assigned_at: chrono::Utc::now(),
    });

    coordinator_project.save(db).await?;
    Ok(coordinator_project)
}

/// Get all coordinators for a project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<CoordinatorProject>>> {
    CoordinatorProject::where_col(|cp| cp.project_id.equal(project_id))
        .run(db)
        .await
}

/// Get all projects assigned to a coordinator
pub(crate) async fn get_projects_by_coordinator(
    db: &PostgresClient, admin_id: i32,
) -> welds::errors::Result<Vec<i32>> {
    let assignments = CoordinatorProject::where_col(|cp| cp.admin_id.equal(admin_id))
        .run(db)
        .await?;

    Ok(assignments
        .into_iter()
        .map(|state| state.project_id)
        .collect())
}

/// Check if a coordinator is assigned to a project
pub(crate) async fn is_assigned(
    db: &PostgresClient, admin_id: i32, project_id: i32,
) -> welds::errors::Result<bool> {
    let assignments = CoordinatorProject::where_col(|cp| cp.admin_id.equal(admin_id))
        .where_col(|cp| cp.project_id.equal(project_id))
        .run(db)
        .await?;

    Ok(!assignments.is_empty())
}

/// Delete a coordinator-project assignment
pub(crate) async fn delete(
    db: &PostgresClient, admin_id: i32, project_id: i32,
) -> welds::errors::Result<()> {
    CoordinatorProject::where_col(|cp| cp.admin_id.equal(admin_id))
        .where_col(|cp| cp.project_id.equal(project_id))
        .delete(db)
        .await?;

    Ok(())
}
