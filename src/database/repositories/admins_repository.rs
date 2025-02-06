use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::admins::ActiveModel;
use entity::admins::Entity;
use log::info;
use password_auth::generate_hash;
use rand::distr::Alphanumeric;
use rand::Rng;
use repository_macro::RepositoryMethods;
use sea_orm::{ActiveValue, DatabaseConnection};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct AdminsRepository {
    db_conn: DatabaseConnection,
}

impl AdminsRepository {
    pub(crate) async fn create_default_admin(&self) {
        let password = generate_random_string(8);
        let username = "admin".to_string();

        self.create(ActiveModel {
            id: ActiveValue::NotSet,
            username: ActiveValue::Set(username.clone()),
            email: ActiveValue::NotSet,
            password_hash: ActiveValue::Set(generate_hash(&password).into_bytes()),
        })
        .await
        .expect("Failed to create default admin");

        info!(
            "Admin created\nUsername: {}\nPassword: {}",
            username, password
        );
    }
}

#[inline(always)]
fn generate_random_string(length: usize) -> String {
    let rng = rand::rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
