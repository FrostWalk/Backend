use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(pk_auto(Projects::ProjectId))
                    .col(string(Projects::Name).not_null())
                    .col(integer(Projects::Year).not_null())
                    .col(integer(Projects::MaxStudentUploads).not_null())
                    .col(integer(Projects::MaxGroupSize).not_null())
                    .col(boolean(Projects::Active).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Projects {
    Table,
    ProjectId,
    Name,
    Year,
    Active,
    MaxGroupSize,
    MaxStudentUploads,
}
