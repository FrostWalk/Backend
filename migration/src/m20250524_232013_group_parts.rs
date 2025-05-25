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
                    .table(GroupParts::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupParts::GroupPartId))
                    .col(integer(GroupParts::ProjectId).not_null())
                    .col(string(GroupParts::Name).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupParts::Table, GroupParts::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .table(GroupParts::Table)
                            .col(GroupParts::ProjectId)
                            .col(GroupParts::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupParts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum GroupParts {
    Table,
    GroupPartId,
    ProjectId,
    Name,
}
