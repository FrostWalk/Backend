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

/// Get a security code by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, security_code_id: i32,
) -> welds::errors::Result<Option<DbState<SecurityCode>>> {
    let mut rows = SecurityCode::where_col(|sc| sc.security_code_id.equal(security_code_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Update a security code
pub(crate) async fn update(
    db: &PostgresClient, security_code_id: i32, code: String,
    expiration: chrono::DateTime<chrono::Utc>,
) -> welds::errors::Result<Option<DbState<SecurityCode>>> {
    let mut security_code =
        SecurityCode::where_col(|sc| sc.security_code_id.equal(security_code_id))
            .run(db)
            .await?;

    if let Some(mut code_state) = security_code.pop() {
        code_state.code = code;
        code_state.expiration = expiration;
        code_state.save(db).await?;
        Ok(Some(code_state))
    } else {
        Ok(None)
    }
}

/// Delete a security code
pub(crate) async fn delete(
    db: &PostgresClient, security_code_id: i32,
) -> welds::errors::Result<()> {
    SecurityCode::where_col(|sc| sc.security_code_id.equal(security_code_id))
        .delete(db)
        .await?;

    Ok(())
}
