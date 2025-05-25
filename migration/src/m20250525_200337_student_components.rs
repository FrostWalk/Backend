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
                    .table(StudentsComponents::Table)
                    .if_not_exists()
                    .col(pk_auto(StudentsComponents::StudentsComponentId))
                    .col(integer(StudentsComponents::ProjectId).not_null())
                    .col(string(StudentsComponents::Name).not_null())
                    .index(
                        Index::create()
                            .table(StudentsComponents::Table)
                            .col(StudentsComponents::ProjectId)
                            .col(StudentsComponents::Name)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StudentsComponents::Table, StudentsComponents::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentsComponents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum StudentsComponents {
    Table,
    StudentsComponentId,
    ProjectId,
    Name,
}
