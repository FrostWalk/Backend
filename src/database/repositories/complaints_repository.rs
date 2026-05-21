use crate::models::complaint::Complaint;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn create(
    db: &PostgresClient, complaint: Complaint,
) -> welds::errors::Result<DbState<Complaint>> {
    let mut state = DbState::new_uncreated(complaint);
    state.save(db).await?;
    Ok(state)
}

pub(crate) async fn get_filed_by_group(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Vec<DbState<Complaint>>> {
    Complaint::where_col(|c| c.from_group_id.equal(group_id))
        .run(db)
        .await
}

pub(crate) async fn get_received_by_group(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Vec<DbState<Complaint>>> {
    Complaint::where_col(|c| c.to_group_id.equal(group_id))
        .run(db)
        .await
}
