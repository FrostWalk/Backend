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
                    .table(Complaints::Table)
                    .if_not_exists()
                    .col(pk_auto(Complaints::ComplaintId))
                    .col(integer(Complaints::FromGroupId).not_null())
                    .col(integer(Complaints::ToGroupId).not_null())
                    .col(text(Complaints::Text).not_null())
                    .col(timestamp(Complaints::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Complaints::Table, Complaints::FromGroupId)
                            .to(Groups::Table, Groups::GroupId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Complaints::Table, Complaints::ToGroupId)
                            .to(Groups::Table, Groups::GroupId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Complaints::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Complaints {
    Table,
    ComplaintId,
    FromGroupId,
    ToGroupId,
    Text,
    CreatedAt,
}
