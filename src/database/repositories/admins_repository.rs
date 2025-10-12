use crate::models::admin::Admin;
use crate::models::admin_role::{AdminRole, AvailableAdminRole};
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

    match seed_admin_roles(db).await {
        Ok(_) => {}
        Err(e) => {
            panic!("unable to create admin roles {e}");
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

async fn seed_admin_roles(db: &impl welds::Client) -> welds::errors::Result<()> {
    let roles: &[(i32, &str)] = &[
        (AvailableAdminRole::Root as i32, "Root"),
        (AvailableAdminRole::Professor as i32, "Professor"),
        (AvailableAdminRole::Coordinator as i32, "Coordinator"),
    ];

    for (id, name) in roles {
        let mut rows = AdminRole::where_col(|r| r.admin_role_id.equal(*id))
            .limit(1)
            .run(db)
            .await?;

        if let Some(mut state) = rows.pop() {
            if state.name != *name {
                state.name = (*name).to_string();
                state.save(db).await?;
            }
        } else {
            // not exists: insert
            let mut state = DbState::new_uncreated(AdminRole {
                admin_role_id: *id,
                name: (*name).to_string(),
            });
            state.save(db).await?;
        }
    }

    Ok(())
}
