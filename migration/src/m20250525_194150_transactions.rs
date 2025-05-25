use crate::m20250524_222426_groups::Groups;
use crate::m20250524_225515_fairs::Fairs;
use crate::m20250524_232050_group_part_selections::GroupPartSelections;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(pk_auto(Transactions::TransactionId))
                    .col(integer(Transactions::BuyerGroupId).not_null())
                    .col(integer(Transactions::GroupPartSelectionId).not_null())
                    .col(integer(Transactions::FairId).not_null())
                    .col(timestamp(Transactions::Timestamp).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Transactions::Table, Transactions::BuyerGroupId)
                            .to(Groups::Table, Groups::GroupId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Transactions::Table, Transactions::GroupPartSelectionId)
                            .to(
                                GroupPartSelections::Table,
                                GroupPartSelections::GroupPartSelectionId,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Transactions::Table, Transactions::FairId)
                            .to(Fairs::Table, Fairs::FairId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    TransactionId,
    BuyerGroupId,
    GroupPartSelectionId,
    FairId,
    Timestamp,
}
