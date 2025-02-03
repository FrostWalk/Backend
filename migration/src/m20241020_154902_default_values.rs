use crate::m20220101_000001_create_tables::{AuxiliaryRoles, Roles};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Roles::Table)
                    .columns([Roles::Id, Roles::Name])
                    .values_panic([1.into(), "Professor".into()])
                    .values_panic([2.into(), "Tutor".into()])
                    .values_panic([3.into(), "Working group coordinator".into()])
                    .values_panic([4.into(), "Group leader".into()])
                    .values_panic([5.into(), "Group member".into()])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(AuxiliaryRoles::Table)
                    .columns([AuxiliaryRoles::Id, AuxiliaryRoles::Name])
                    .values_panic([1.into(), "Git maintainer".into()])
                    .values_panic([2.into(), "Tester".into()])
                    .values_panic([3.into(), "Reporter".into()])
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
