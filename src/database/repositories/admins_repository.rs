use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::admins;
use entity::admins::ActiveModel;
use entity::admins::Entity;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use password_auth::generate_hash;
use repository_macro::RepositoryMethods;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr};
use std::ops::Deref;

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct AdminsRepository {
    db_conn: DatabaseConnection,
}

impl AdminsRepository {
    pub(crate) async fn create_default_admin(&self, email: &String, password: &String) {
        self.create(ActiveModel {
            admin_id: ActiveValue::NotSet,
            first_name: ActiveValue::Set("admin".to_string()),
            last_name: ActiveValue::Set("".to_string()),
            email: ActiveValue::Set(email.deref().to_owned()),
            password_hash: ActiveValue::Set(generate_hash(password)),
            admin_role_id: ActiveValue::Set(AdminRole::Root.into()),
        })
        .await
        .expect("Failed to create default admin");
    }

    pub(crate) async fn get_from_mail(
        &self, mail: &String,
    ) -> Result<Option<admins::Model>, DbErr> {
        self.get_one_from_filter(admins::Column::Email.eq(mail))
            .await
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
