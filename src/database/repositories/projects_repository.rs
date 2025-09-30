use crate::models::project::Project;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all projects from the database
pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Project>>> {
    Project::all().run(db).await
}

/// Get a project by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Option<DbState<Project>>> {
    let mut rows = Project::where_col(|p| p.project_id.equal(project_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Delete a project by its ID
/// Returns true if the project was deleted, false if not found
pub(crate) async fn delete_by_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<bool> {
    let mut rows = Project::where_col(|p| p.project_id.equal(project_id))
        .run(db)
        .await?;

    if let Some(mut state) = rows.pop() {
        state.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Update a project
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<Project>,
) -> welds::errors::Result<DbState<Project>> {
    state.save(db).await?;
    Ok(state)
}
