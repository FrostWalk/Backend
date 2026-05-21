use crate::models::fair::Fair;
use chrono::Utc;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn create(
    db: &PostgresClient, fair: Fair,
) -> welds::errors::Result<DbState<Fair>> {
    let mut state = DbState::new_uncreated(fair);
    state.save(db).await?;
    Ok(state)
}

pub(crate) async fn get_by_id(
    db: &PostgresClient, fair_id: i32,
) -> welds::errors::Result<Option<DbState<Fair>>> {
    let mut rows = Fair::where_col(|f| f.fair_id.equal(fair_id))
        .run(db)
        .await?;
    Ok(rows.pop())
}

pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Option<DbState<Fair>>> {
    let mut rows = Fair::where_col(|f| f.project_id.equal(project_id))
        .run(db)
        .await?;
    Ok(rows.pop())
}

pub(crate) async fn update(
    db: &PostgresClient, state: &mut DbState<Fair>,
) -> welds::errors::Result<()> {
    state.save(db).await
}

pub(crate) async fn enable(
    db: &PostgresClient, fair_id: i32,
) -> welds::errors::Result<Option<DbState<Fair>>> {
    let mut rows = Fair::where_col(|f| f.fair_id.equal(fair_id))
        .run(db)
        .await?;
    if let Some(mut state) = rows.pop() {
        state.start_date = Utc::now();
        state.save(db).await?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

pub(crate) async fn disable(
    db: &PostgresClient, fair_id: i32,
) -> welds::errors::Result<Option<DbState<Fair>>> {
    let mut rows = Fair::where_col(|f| f.fair_id.equal(fair_id))
        .run(db)
        .await?;
    if let Some(mut state) = rows.pop() {
        state.end_date = Utc::now();
        state.save(db).await?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

pub(crate) fn is_active(fair: &Fair) -> bool {
    let now = Utc::now();
    fair.start_date <= now && now <= fair.end_date
}
