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
                    .table(Fairs::Table)
                    .if_not_exists()
                    .col(pk_auto(Fairs::FairId))
                    .col(integer(Fairs::ProjectId).not_null())
                    .col(string(Fairs::Details).not_null())
                    .col(timestamp(Fairs::StartDate).not_null())
                    .col(timestamp(Fairs::EndDate).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Fairs::Table, Fairs::ProjectId)
                            .to(Projects::Table, Projects::ProjectId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fairs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Fairs {
    Table,
    FairId,
    ProjectId,
    Details,
    StartDate,
    EndDate,
}
