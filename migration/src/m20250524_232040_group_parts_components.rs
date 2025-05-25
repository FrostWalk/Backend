use crate::m20250524_232013_group_parts::GroupParts;
use crate::m20250524_232024_group_components::GroupComponents;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupPartsComponents::Table)
                    .if_not_exists()
                    .col(integer(GroupPartsComponents::GroupComponentId).not_null())
                    .col(integer(GroupPartsComponents::GroupPartId).not_null())
                    .col(integer(GroupPartsComponents::Quantity).not_null())
                    .primary_key(
                        Index::create()
                            .table(GroupPartsComponents::Table)
                            .col(GroupPartsComponents::GroupComponentId)
                            .col(GroupPartsComponents::GroupPartId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                GroupPartsComponents::Table,
                                GroupPartsComponents::GroupComponentId,
                            )
                            .to(GroupComponents::Table, GroupComponents::GroupComponentId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                GroupPartsComponents::Table,
                                GroupPartsComponents::GroupPartId,
                            )
                            .to(GroupParts::Table, GroupParts::GroupPartId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupPartsComponents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GroupPartsComponents {
    Table,
    GroupPartId,
    GroupComponentId,
    Quantity,
}
