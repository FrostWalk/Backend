use crate::models::security_code::SecurityCode;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Check if a security code exists
pub(crate) async fn security_code_exists(
    db: &PostgresClient, code: &str,
) -> welds::errors::Result<bool> {
    let rows = SecurityCode::where_col(|sc| sc.code.equal(code))
        .limit(1)
        .run(db)
        .await?;
    Ok(!rows.is_empty())
}

/// Get all security codes
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<SecurityCode>>> {
    SecurityCode::all().run(db).await
}

/// Get a security code by its code string
pub(crate) async fn get_by_code(
    db: &PostgresClient, code: &str,
) -> welds::errors::Result<Option<DbState<SecurityCode>>> {
    let mut rows = SecurityCode::where_col(|sc| sc.code.equal(code))
        .run(db)
        .await?;

    Ok(rows.pop())
}
