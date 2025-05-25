use crate::m20250524_184046_projects::Projects;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentParts::Table)
                    .if_not_exists()
                    .col(pk_auto(StudentParts::StudentPartId))
                    .col(integer(StudentParts::ProjectId).not_null())
                    .col(string(StudentParts::Name).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(StudentParts::Table, StudentParts::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .table(StudentParts::Table)
                            .col(StudentParts::ProjectId)
                            .col(StudentParts::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentParts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum StudentParts {
    Table,
    StudentPartId,
    ProjectId,
    Name,
}
