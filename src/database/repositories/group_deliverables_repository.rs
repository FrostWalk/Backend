use crate::models::group_deliverable::GroupDeliverable;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all group deliverables
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<GroupDeliverable>>> {
    GroupDeliverable::all().run(db).await
}

/// Get a group deliverable by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, group_deliverable_id: i32,
) -> welds::errors::Result<Option<DbState<GroupDeliverable>>> {
    let mut rows =
        GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(group_deliverable_id))
            .run(db)
            .await?;

    Ok(rows.pop())
}

/// Get all group deliverables for a specific project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupDeliverable>>> {
    GroupDeliverable::where_col(|gd| gd.project_id.equal(project_id))
        .run(db)
        .await
}

/// Check if a group deliverable with the same name exists in a project (excluding a specific ID)
pub(crate) async fn check_name_exists_excluding(
    db: &PostgresClient, project_id: i32, name: &str, excluding_id: i32,
) -> welds::errors::Result<bool> {
    let rows = GroupDeliverable::where_col(|gd| gd.project_id.equal(project_id))
        .where_col(|gd| gd.name.equal(name))
        .where_col(|gd| gd.group_deliverable_id.not_equal(excluding_id))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Check if a group deliverable with the same name exists in a project
pub(crate) async fn check_name_exists(
    db: &PostgresClient, project_id: i32, name: &str,
) -> welds::errors::Result<bool> {
    let rows = GroupDeliverable::where_col(|gd| gd.project_id.equal(project_id))
        .where_col(|gd| gd.name.equal(name))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Create a new group deliverable
pub(crate) async fn create(
    db: &PostgresClient, group_deliverable: GroupDeliverable,
) -> welds::errors::Result<DbState<GroupDeliverable>> {
    let mut state = DbState::new_uncreated(group_deliverable);
    state.save(db).await?;
    Ok(state)
}

/// Delete a group deliverable by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, group_deliverable_id: i32,
) -> welds::errors::Result<()> {
    GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(group_deliverable_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Update a group deliverable by ID
pub(crate) async fn update_by_id(
    db: &PostgresClient, group_deliverable_id: i32, name: &str,
) -> welds::errors::Result<()> {
    GroupDeliverable::where_col(|gd| gd.group_deliverable_id.equal(group_deliverable_id))
        .set(|gd| gd.name, name)
        .run(db)
        .await?;
    Ok(())
}
