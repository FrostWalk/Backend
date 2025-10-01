use crate::models::group_deliverable::GroupDeliverable;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

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
