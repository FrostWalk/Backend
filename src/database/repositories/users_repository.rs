use crate::database::repository_methods_trait::RepositoryMethods;
use derive_new::new;
use entity::students;
use entity::students::ActiveModel;
use entity::students::Entity;
use entity::students::Model;
use repository_macro::RepositoryMethods;
use sea_orm::ColumnTrait;
use sea_orm::{DatabaseConnection, DbErr};

#[derive(new, RepositoryMethods, Clone)]
pub(crate) struct StudentsRepository {
    db_conn: DatabaseConnection,
}

impl StudentsRepository {
    pub(crate) async fn get_from_mail(&self, mail: &String) -> Result<Option<Model>, DbErr> {
        self.get_one_from_filter(students::Column::Email.eq(mail))
            .await
    }
}
