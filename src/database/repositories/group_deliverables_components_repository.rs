use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::group_deliverables_component::GroupDeliverablesComponent;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all group deliverables components relationships
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<GroupDeliverablesComponent>>> {
    GroupDeliverablesComponent::all().run(db).await
}

/// Get a group deliverables component relationship by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, id: i32,
) -> welds::errors::Result<Option<DbState<GroupDeliverablesComponent>>> {
    let mut rows = GroupDeliverablesComponent::where_col(|gdc| gdc.id.equal(id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Get all components for a specific group deliverable
pub(crate) async fn get_by_deliverable_id(
    db: &PostgresClient, deliverable_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupDeliverablesComponent>>> {
    GroupDeliverablesComponent::where_col(|gdc| gdc.group_deliverable_id.equal(deliverable_id))
        .run(db)
        .await
}

/// Get all deliverables for a specific component
pub(crate) async fn get_by_component_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupDeliverablesComponent>>> {
    GroupDeliverablesComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await
}

/// Check if a relationship exists between a deliverable and component
pub(crate) async fn relationship_exists(
    db: &PostgresClient, deliverable_id: i32, component_id: i32,
) -> welds::errors::Result<bool> {
    let rows =
        GroupDeliverablesComponent::where_col(|gdc| gdc.group_deliverable_id.equal(deliverable_id))
            .where_col(|gdc| gdc.group_deliverable_component_id.equal(component_id))
            .limit(1)
            .run(db)
            .await?;

    Ok(!rows.is_empty())
}

/// Check if component is part of a deliverable
pub(crate) async fn is_component_in_deliverable(
    db: &PostgresClient, deliverable_id: i32, component_id: i32,
) -> welds::errors::Result<bool> {
    relationship_exists(db, deliverable_id, component_id).await
}

/// Create a new group deliverables component relationship
pub(crate) async fn create(
    db: &PostgresClient, group_deliverables_component: GroupDeliverablesComponent,
) -> welds::errors::Result<DbState<GroupDeliverablesComponent>> {
    let mut state = DbState::new_uncreated(group_deliverables_component);
    state.save(db).await?;
    Ok(state)
}

/// Get components with their details for a specific group deliverable
pub(crate) async fn get_components_with_details_for_deliverable(
    db: &PostgresClient, deliverable_id: i32,
) -> welds::errors::Result<
    Vec<(
        DbState<GroupDeliverablesComponent>,
        DbState<GroupDeliverableComponent>,
    )>,
> {
    let relationships =
        GroupDeliverablesComponent::where_col(|gdc| gdc.group_deliverable_id.equal(deliverable_id))
            .run(db)
            .await?;

    let mut result = Vec::new();
    for relationship in relationships {
        let mut components = GroupDeliverableComponent::where_col(|gc| {
            gc.group_deliverable_component_id
                .equal(relationship.group_deliverable_component_id)
        })
        .run(db)
        .await?;

        if let Some(component) = components.pop() {
            result.push((relationship, component));
        }
    }

    Ok(result)
}

/// Get deliverables with their details for a specific group component
pub(crate) async fn get_deliverables_with_details_for_component(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<
    Vec<(
        DbState<GroupDeliverablesComponent>,
        DbState<GroupDeliverable>,
    )>,
> {
    let relationships = GroupDeliverablesComponent::where_col(|gdc| {
        gdc.group_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await?;

    let mut result = Vec::new();
    for relationship in relationships {
        let mut deliverables = GroupDeliverable::where_col(|gd| {
            gd.group_deliverable_id
                .equal(relationship.group_deliverable_id)
        })
        .run(db)
        .await?;

        if let Some(deliverable) = deliverables.pop() {
            result.push((relationship, deliverable));
        }
    }

    Ok(result)
}
/// Delete a group deliverables component relationship by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, relationship_id: i32,
) -> welds::errors::Result<()> {
    GroupDeliverablesComponent::where_col(|gdc| gdc.id.equal(relationship_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Update a group deliverables component relationship
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<GroupDeliverablesComponent>,
) -> welds::errors::Result<DbState<GroupDeliverablesComponent>> {
    state.save(db).await?;
    Ok(state)
}
