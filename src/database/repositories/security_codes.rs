use crate::models::security_code::SecurityCode;
use welds::connections::postgres::PostgresClient;

pub(crate) async fn security_code_exists(
    db: &PostgresClient, code: &str,
) -> welds::errors::Result<bool> {
    let rows = SecurityCode::where_col(|sc| sc.code.equal(code))
        .limit(1)
        .run(db)
        .await?;
    Ok(!rows.is_empty())
}
