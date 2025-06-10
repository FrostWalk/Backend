use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentRoles::Table)
                    .if_not_exists()
                    .col(pk_auto(StudentRoles::StudentRoleId))
                    .col(string(StudentRoles::Name).not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(StudentRoles::Table)
                    .columns([StudentRoles::StudentRoleId, StudentRoles::Name])
                    .values_panic([1.into(), "Member".into()])
                    .values_panic([2.into(), "Leader".into()])
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum StudentRoles {
    Table,
    StudentRoleId,
    Name,
}
