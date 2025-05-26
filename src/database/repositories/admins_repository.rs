use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::admins::ActiveModel;
use entity::admins::Entity;
use password_auth::generate_hash;
use repository_macro::RepositoryMethods;
use sea_orm::{ActiveValue, DatabaseConnection};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct AdminsRepository {
    db_conn: DatabaseConnection,
}

impl AdminsRepository {
    pub(crate) async fn create_default_admin(&self, email: String, password: String) {
        self.create(ActiveModel {
            admin_id: ActiveValue::NotSet,
            first_name: ActiveValue::Set("admin".to_string()),
            last_name: ActiveValue::Set("".to_string()),
            email: ActiveValue::Set(email),
            password_hash: ActiveValue::Set(generate_hash(&password)),
            admin_role_id: ActiveValue::Set(AdminRole::Root.into()),
        })
        .await
        .expect("Failed to create default admin");
    }
}
pub(crate) enum AdminRole {
    Root = 1,
    Professor = 2,
    Tutor = 3,
    Coordinator = 4,
}

impl Into<i32> for AdminRole {
    fn into(self) -> i32 {
        self as i32
    }
}
