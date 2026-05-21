use crate::models::transaction::Transaction;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn create(
    db: &PostgresClient, transaction: Transaction,
) -> welds::errors::Result<DbState<Transaction>> {
    let mut state = DbState::new_uncreated(transaction);
    state.save(db).await?;
    Ok(state)
}

pub(crate) async fn get_by_id(
    db: &PostgresClient, transaction_id: i32,
) -> welds::errors::Result<Option<DbState<Transaction>>> {
    let mut rows = Transaction::where_col(|t| t.transaction_id.equal(transaction_id))
        .run(db)
        .await?;
    Ok(rows.pop())
}

pub(crate) async fn get_by_fair_and_buyer(
    db: &PostgresClient, fair_id: i32, buyer_group_id: i32,
) -> welds::errors::Result<Vec<DbState<Transaction>>> {
    Transaction::where_col(|t| t.fair_id.equal(fair_id))
        .where_col(|t| t.buyer_group_id.equal(buyer_group_id))
        .run(db)
        .await
}

/// Count distinct (component, seller_group) pairs purchased by a group in a fair.
/// Used to check the min_purchases requirement.
pub(crate) async fn count_distinct_purchases(
    db: &PostgresClient, fair_id: i32, buyer_group_id: i32,
) -> sqlx::Result<i64> {
    let row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT (t.group_deliverable_selection_id, t.group_deliverable_component_id))
        FROM transactions t
        WHERE t.fair_id = $1 AND t.buyer_group_id = $2
        "#,
    )
    .bind(fair_id)
    .bind(buyer_group_id)
    .fetch_one(db.as_sqlx_pool())
    .await?;
    Ok(row.0.unwrap_or(0))
}

/// Check whether a specific (buyer, seller_selection, component) purchase already exists.
pub(crate) async fn purchase_exists(
    db: &PostgresClient, buyer_group_id: i32, group_deliverable_selection_id: i32,
    group_deliverable_component_id: i32,
) -> welds::errors::Result<bool> {
    let rows = Transaction::where_col(|t| t.buyer_group_id.equal(buyer_group_id))
        .where_col(|t| {
            t.group_deliverable_selection_id
                .equal(group_deliverable_selection_id)
        })
        .where_col(|t| {
            t.group_deliverable_component_id
                .equal(group_deliverable_component_id)
        })
        .run(db)
        .await?;
    Ok(!rows.is_empty())
}
