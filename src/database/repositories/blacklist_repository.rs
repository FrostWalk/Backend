use crate::models::blacklist::Blacklist;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all blacklist entries.
pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Blacklist>>> {
    Blacklist::all().run(db).await
}

/// Get blacklist entry by primary key.
pub(crate) async fn get_by_id(
    db: &PostgresClient, blacklist_id: i32,
) -> welds::errors::Result<Option<DbState<Blacklist>>> {
    let mut rows = Blacklist::where_col(|b| b.blacklist_id.equal(blacklist_id))
        .run(db)
        .await?;
    Ok(rows.pop())
}

/// Get blacklist entry by university id.
pub(crate) async fn get_by_university_id(
    db: &PostgresClient, university_id: i32,
) -> welds::errors::Result<Option<DbState<Blacklist>>> {
    let mut rows = Blacklist::where_col(|b| b.university_id.equal(university_id))
        .run(db)
        .await?;
    Ok(rows.pop())
}

/// Create blacklist entry.
pub(crate) async fn create(
    db: &PostgresClient, entry: Blacklist,
) -> welds::errors::Result<DbState<Blacklist>> {
    let mut state = DbState::new_uncreated(entry);
    state.save(db).await?;
    Ok(state)
}

/// Partially update blacklist entry fields.
pub(crate) async fn update_by_id(
    db: &PostgresClient, blacklist_id: i32, description: Option<String>,
    first_name: Option<String>, last_name: Option<String>,
) -> welds::errors::Result<Option<DbState<Blacklist>>> {
    let existing = get_by_id(db, blacklist_id).await?;

    let Some(mut state) = existing else {
        return Ok(None);
    };

    if let Some(value) = description {
        state.as_mut().description = value;
    }
    if let Some(value) = first_name {
        state.as_mut().first_name = value;
    }
    if let Some(value) = last_name {
        state.as_mut().last_name = value;
    }

    state.save(db).await?;
    Ok(Some(state))
}

/// Delete blacklist entry by id.
pub(crate) async fn delete_by_id(
    db: &PostgresClient, blacklist_id: i32,
) -> welds::errors::Result<bool> {
    let existing = get_by_id(db, blacklist_id).await?;

    let Some(mut state) = existing else {
        return Ok(false);
    };

    state.delete(db).await?;
    Ok(true)
}
