use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Blacklist::Table)
                    .if_not_exists()
                    .col(pk_auto(Blacklist::BlacklistId))
                    .col(integer(Blacklist::UniversityId).not_null().unique_key())
                    .col(text(Blacklist::Description).not_null())
                    .col(string(Blacklist::FistName).not_null())
                    .col(string(Blacklist::LastName).not_null())
                    .col(timestamp(Blacklist::BannedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Blacklist::Table).to_owned())
            .await
    }
}
#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Blacklist {
    Table,
    BlacklistId,
    UniversityId,
    BannedAt,
    Description,
    FistName,
    LastName,
}
