use crate::database::seed::seed_all_roles;
use crate::models::admin::Admin;
use crate::models::admin_role::AvailableAdminRole;
use log::{error, info};
use password_auth::generate_hash;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Admin>>> {
    Admin::all().run(db).await
}

/// Get an admin by email
pub(crate) async fn get_by_email(
    db: &PostgresClient, email: &str,
) -> welds::errors::Result<Option<DbState<Admin>>> {
    let mut rows = Admin::where_col(|a| a.email.equal(email)).run(db).await?;

    Ok(rows.pop())
}

/// Get an admin by ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, admin_id: i32,
) -> welds::errors::Result<Option<DbState<Admin>>> {
    let mut rows = Admin::where_col(|a| a.admin_id.equal(admin_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Delete an admin by ID
/// Returns true if the admin was deleted, false if not found
pub(crate) async fn delete_by_id(
    db: &PostgresClient, admin_id: i32,
) -> welds::errors::Result<bool> {
    let mut rows = Admin::where_col(|a| a.admin_id.equal(admin_id))
        .run(db)
        .await?;

    if let Some(mut state) = rows.pop() {
        state.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Update an admin's password by email
pub(crate) async fn update_password_by_email(
    db: &PostgresClient, email: &str, password_hash: String,
) -> welds::errors::Result<()> {
    Admin::where_col(|a| a.email.equal(email))
        .set(|a| a.password_hash, password_hash)
        .run(db)
        .await?;
    Ok(())
}

/// Create a new admin
pub(crate) async fn create(
    db: &PostgresClient, admin: Admin,
) -> welds::errors::Result<DbState<Admin>> {
    let mut state = DbState::new_uncreated(admin);
    state.save(db).await?;
    Ok(state)
}

/// Update an admin by ID
pub(crate) async fn update_by_id(
    db: &PostgresClient, admin_id: i32, first_name: Option<String>, last_name: Option<String>,
    email: Option<String>, password_hash: Option<String>,
) -> welds::errors::Result<()> {
    if let Some(name) = first_name {
        Admin::where_col(|a| a.admin_id.equal(admin_id))
            .set(|a| a.first_name, name)
            .run(db)
            .await?;
    }
    if let Some(name) = last_name {
        Admin::where_col(|a| a.admin_id.equal(admin_id))
            .set(|a| a.last_name, name)
            .run(db)
            .await?;
    }
    if let Some(email) = email {
        Admin::where_col(|a| a.admin_id.equal(admin_id))
            .set(|a| a.email, email)
            .run(db)
            .await?;
    }
    if let Some(hash) = password_hash {
        Admin::where_col(|a| a.admin_id.equal(admin_id))
            .set(|a| a.password_hash, hash)
            .run(db)
            .await?;
    }

    Ok(())
}

pub(crate) async fn create_default_admin(db: &PostgresClient, email: String, password: String) {
    let found = match get_all(db).await {
        Ok(v) => v.len(),
        Err(e) => {
            error!("unable to find admins: {}", e);
            0
        }
    };
    if found > 0 {
        return;
    }

    match seed_all_roles(db).await {
        Ok(_) => {}
        Err(e) => {
            panic!("unable to seed roles: {e}");
        }
    };

    let mut admin = Admin::new();
    admin.admin_role_id = AvailableAdminRole::Root.into();
    admin.email = email.clone();
    admin.password_hash = generate_hash(password);
    admin.first_name = "root".to_string();
    admin.last_name = String::new();

    info!("creating default admin");
    match admin.save(db).await {
        Ok(_) => {}
        Err(e) => {
            panic!("unable to create default admin {:?} error: {}", admin, e)
        }
    }
}
