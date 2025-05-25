use crate::m20250524_184046_projects::Projects;
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
                    .table(SecurityCodes::Table)
                    .if_not_exists()
                    .col(pk_auto(SecurityCodes::SecurityCodeId))
                    .col(integer(SecurityCodes::ProjectId).not_null())
                    .col(integer(SecurityCodes::StudentRoleId).not_null())
                    .col(string(SecurityCodes::Code).not_null().unique_key())
                    .col(timestamp(SecurityCodes::Expiration).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(SecurityCodes::Table, SecurityCodes::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SecurityCodes::Table, SecurityCodes::StudentRoleId)
                            .to(StudentRoles::Table, StudentRoles::StudentRoleId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SecurityCodes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SecurityCodes {
    Table,
    SecurityCodeId,
    ProjectId,
    Code,
    StudentRoleId,
    Expiration,
}
