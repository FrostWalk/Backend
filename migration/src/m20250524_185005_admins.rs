use crate::m20250524_184935_admin_roles::AdminRoles;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Admins::Table)
                    .if_not_exists()
                    .col(pk_auto(Admins::AdminId))
                    .col(string(Admins::FirstName).not_null())
                    .col(string(Admins::LastName).not_null())
                    .col(string(Admins::Email).not_null().unique_key())
                    .col(string(Admins::PasswordHash).not_null())
                    .col(small_integer(Admins::AdminRoleId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Admins::Table, Admins::AdminRoleId)
                            .to(AdminRoles::Table, AdminRoles::AdminRoleId)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Admins::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Admins {
    Table,
    AdminId,
    Email,
    FirstName,
    LastName,
    PasswordHash,
    AdminRoleId,
}
