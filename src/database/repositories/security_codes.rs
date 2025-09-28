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

pub(crate) async fn get_student_role_id_from_security_code(
    db: &PostgresClient, code: &str,
) -> welds::errors::Result<Option<i32>> {
    let rows = SecurityCode::where_col(|sc| sc.code.equal(code))
        .limit(1)
        .run(db)
        .await?;

    if let Some(security_code) = rows.first() {
        Ok(Some(security_code.student_role_id))
    } else {
        Ok(None)
    }
}
