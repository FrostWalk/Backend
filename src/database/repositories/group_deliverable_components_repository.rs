use crate::models::group_deliverable_component::GroupDeliverableComponent;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all group deliverable components
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<GroupDeliverableComponent>>> {
    GroupDeliverableComponent::all().run(db).await
}

/// Get a group deliverable component by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<Option<DbState<GroupDeliverableComponent>>> {
    let mut rows = GroupDeliverableComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await?;

    Ok(rows.pop())
}

/// Get all group deliverable components for a specific project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupDeliverableComponent>>> {
    GroupDeliverableComponent::where_col(|gdc| gdc.project_id.equal(project_id))
        .run(db)
        .await
}

/// Check if a group component with the same name exists in a project (excluding a specific ID)
pub(crate) async fn check_name_exists_excluding(
    db: &PostgresClient, project_id: i32, name: &str, excluding_id: i32,
) -> welds::errors::Result<bool> {
    let rows = GroupDeliverableComponent::where_col(|gdc| gdc.project_id.equal(project_id))
        .where_col(|gdc| gdc.name.equal(name))
        .where_col(|gdc| gdc.group_deliverable_component_id.not_equal(excluding_id))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Get component by ID
pub(crate) async fn get_component_by_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<Option<DbState<GroupDeliverableComponent>>> {
    let mut rows = GroupDeliverableComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await?;

    Ok(rows.pop())
}

/// Create a new group deliverable component
pub(crate) async fn create(
    db: &PostgresClient, group_deliverable_component: GroupDeliverableComponent,
) -> welds::errors::Result<DbState<GroupDeliverableComponent>> {
    let mut state = DbState::new_uncreated(group_deliverable_component);
    state.save(db).await?;
    Ok(state)
}

/// Update a group deliverable component
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<GroupDeliverableComponent>,
) -> welds::errors::Result<DbState<GroupDeliverableComponent>> {
    state.save(db).await?;
    Ok(state)
}

/// Check if a group component with the same name exists in a project
pub(crate) async fn check_name_exists(
    db: &PostgresClient, project_id: i32, name: &str,
) -> welds::errors::Result<bool> {
    let rows = GroupDeliverableComponent::where_col(|gdc| gdc.project_id.equal(project_id))
        .where_col(|gdc| gdc.name.equal(name))
        .limit(1)
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Delete a group deliverable component by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<()> {
    GroupDeliverableComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .delete(db)
    .await?;
    Ok(())
}
