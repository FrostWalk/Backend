use crate::models::group_component_implementation_detail::GroupComponentImplementationDetail;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all implementation details for a selection
pub(crate) async fn get_by_selection_id(
    db: &PostgresClient, selection_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupComponentImplementationDetail>>> {
    GroupComponentImplementationDetail::where_col(|gcid| {
        gcid.group_deliverable_selection_id.equal(selection_id)
    })
    .run(db)
    .await
}

/// Get specific component implementation detail
pub(crate) async fn get_by_selection_and_component(
    db: &PostgresClient, selection_id: i32, component_id: i32,
) -> welds::errors::Result<Option<DbState<GroupComponentImplementationDetail>>> {
    let mut rows = GroupComponentImplementationDetail::where_col(|gcid| {
        gcid.group_deliverable_selection_id.equal(selection_id)
    })
    .where_col(|gcid| gcid.group_deliverable_component_id.equal(component_id))
    .run(db)
    .await?;

    Ok(rows.pop())
}

/// Check if implementation details exist for a component
pub(crate) async fn exists(
    db: &PostgresClient, selection_id: i32, component_id: i32,
) -> welds::errors::Result<bool> {
    let detail = get_by_selection_and_component(db, selection_id, component_id).await?;
    Ok(detail.is_some())
}

/// Create implementation details
pub(crate) async fn create(
    db: &PostgresClient, selection_id: i32, component_id: i32, markdown_description: String,
    repository_link: String,
) -> welds::errors::Result<DbState<GroupComponentImplementationDetail>> {
    let mut state = DbState::new_uncreated(GroupComponentImplementationDetail {
        id: 0,
        group_deliverable_selection_id: selection_id,
        group_deliverable_component_id: component_id,
        markdown_description,
        repository_link,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

    state.save(db).await?;
    Ok(state)
}

/// Update implementation details
pub(crate) async fn update(
    db: &PostgresClient, selection_id: i32, component_id: i32, markdown_description: String,
    repository_link: String,
) -> welds::errors::Result<Option<DbState<GroupComponentImplementationDetail>>> {
    let mut detail_state = get_by_selection_and_component(db, selection_id, component_id).await?;

    if let Some(detail_state) = detail_state.as_mut() {
        detail_state.markdown_description = markdown_description;
        detail_state.repository_link = repository_link;
        detail_state.updated_at = chrono::Utc::now();
        detail_state.save(db).await?;
    }

    Ok(detail_state)
}

/// Delete implementation details
pub(crate) async fn delete(
    db: &PostgresClient, selection_id: i32, component_id: i32,
) -> welds::errors::Result<bool> {
    let detail_state = get_by_selection_and_component(db, selection_id, component_id).await?;

    if let Some(mut detail_state) = detail_state {
        detail_state.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
