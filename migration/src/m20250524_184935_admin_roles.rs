use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdminRoles::Table)
                    .if_not_exists()
                    .col(pk_auto(AdminRoles::AdminRoleId))
                    .col(string(AdminRoles::Name).not_null().unique_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdminRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum AdminRoles {
    Table,
    AdminRoleId,
    Name,
}
