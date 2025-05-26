use crate::m20250524_212554_students_roles::StudentRoles;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Students::Table)
                    .if_not_exists()
                    .col(pk_auto(Students::StudentId))
                    .col(string(Students::FirstName).not_null())
                    .col(string(Students::LastName).not_null())
                    .col(string(Students::Email).not_null().unique_key())
                    .col(integer(Students::UniversityId).not_null().unique_key())
                    .col(string(Students::PasswordHash).not_null())
                    .col(integer(Students::StudentRoleId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Students::Table, Students::StudentRoleId)
                            .to(StudentRoles::Table, StudentRoles::StudentRoleId)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Students::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Students {
    Table,
    StudentId,
    Email,
    FirstName,
    LastName,
    PasswordHash,
    UniversityId,
    StudentRoleId,
}
