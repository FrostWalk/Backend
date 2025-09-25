use crate::models::admin::Admin;
use log::{error, info};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use password_auth::generate_hash;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Admin>>> {
    Admin::all().run(db).await
}
pub(crate) async fn get_from_mail(
    db: &PostgresClient, mail: &String,
) -> welds::errors::Result<DbState<Admin>> {
    Admin::where_col(|p| p.email.like(mail)).fetch_one(db).await
}

pub(crate) async fn create_default_admin(db: &PostgresClient, email: String, password: String) {
    let found = match get_all(db).await {
        Ok(v) => v.len(),
        Err(e) => {
            error!("unable to find admins {e}");
            0
        }
    };
    if found > 0 {
        return;
    }

    //todo create admin's roles and student's roles

    let mut admin = Admin::new();
    admin.admin_role_id = AdminRole::Root.into();
    admin.email = email.clone();
    admin.password_hash = generate_hash(password);
    admin.first_name = "root".to_string();
    admin.last_name = String::new();

    info!("Creating default admin");
    match admin.save(db).await {
        Ok(_) => {}
        Err(e) => {
            panic!("Unable to create default admin {:?} error: {e}", admin)
        }
    }
}

#[derive(PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub(crate) enum AdminRole {
    Root = 1,
    Professor = 2,
    Tutor = 3,
    Coordinator = 4,
}
pub(crate) const ALL: [AdminRole; 4] = [
    AdminRole::Root,
    AdminRole::Professor,
    AdminRole::Tutor,
    AdminRole::Coordinator,
];
