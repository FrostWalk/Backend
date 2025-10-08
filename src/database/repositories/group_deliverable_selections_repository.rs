use crate::models::group_deliverable_selection::GroupDeliverableSelection;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get a group deliverable selection by group ID
pub(crate) async fn get_by_group_id(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Option<DbState<GroupDeliverableSelection>>> {
    let mut rows = GroupDeliverableSelection::where_col(|gds| gds.group_id.equal(group_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Check if a group has already selected a deliverable
pub(crate) async fn has_selection(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<bool> {
    let selection = get_by_group_id(db, group_id).await?;
    Ok(selection.is_some())
}

/// Check if a link is already in use by another group
pub(crate) async fn link_exists(db: &PostgresClient, link: &str) -> welds::errors::Result<bool> {
    let rows = GroupDeliverableSelection::where_col(|gds| gds.link.equal(link))
        .run(db)
        .await?;

    Ok(!rows.is_empty())
}

/// Check if a link is in use by another group (excluding the specified group)
pub(crate) async fn link_exists_for_other_group(
    db: &PostgresClient, link: &str, group_id: i32,
) -> welds::errors::Result<bool> {
    let rows = GroupDeliverableSelection::where_col(|gds| gds.link.equal(link))
        .run(db)
        .await?;

    for row_state in rows {
        let row = DbState::into_inner(row_state);
        if row.group_id != group_id {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get all group deliverable selections for a project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupDeliverableSelection>>> {
    // First get all groups for the project
    let groups = crate::models::group::Group::where_col(|g| g.project_id.equal(project_id))
        .run(db)
        .await?;

    let mut selections = Vec::new();
    for group_state in groups {
        let group = DbState::into_inner(group_state);
        if let Some(selection) = get_by_group_id(db, group.group_id).await? {
            selections.push(selection);
        }
    }

    Ok(selections)
}
