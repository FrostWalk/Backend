use crate::m20250524_222426_groups::Groups;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupPartSelections::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupPartSelections::GroupPartSelectionId).not_null())
                    .col(integer(GroupPartSelections::GroupId).not_null())
                    .col(text(GroupPartSelections::Link).not_null().unique_key())
                    .col(text(GroupPartSelections::MarkdownText).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupPartSelections::Table, GroupPartSelections::GroupId)
                            .to(Groups::Table, Groups::GroupId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupPartSelections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum GroupPartSelections {
    Table,
    GroupPartSelectionId,
    GroupId,
    Link,
    MarkdownText,
}
