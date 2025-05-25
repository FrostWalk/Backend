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
                    .table(GroupComponents::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupComponents::GroupComponentId))
                    .col(integer(GroupComponents::ProjectId).not_null())
                    .col(string(GroupComponents::Name).not_null())
                    .index(
                        Index::create()
                            .table(GroupComponents::Table)
                            .col(GroupComponents::ProjectId)
                            .col(GroupComponents::Name)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupComponents::Table, GroupComponents::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupComponents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum GroupComponents {
    Table,
    GroupComponentId,
    ProjectId,
    Name,
}
