use crate::m20220101_000001_create_tables::{Roles, RolesHierarchy};
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
                    .columns([Roles::Id, Roles::Name, Roles::IsAuxiliary])
                    .values_panic([1.into(), "Root".into(), false.into()])
                    .values_panic([2.into(), "Professor".into(), false.into()])
                    .values_panic([3.into(), "Tutor".into(), false.into()])
                    .values_panic([4.into(), "Working group coordinator".into(), false.into()])
                    .values_panic([5.into(), "Group leader".into(), false.into()])
                    .values_panic([6.into(), "Group member".into(), false.into()])
                    .values_panic([7.into(), "Git maintainer".into(), true.into()])
                    .values_panic([8.into(), "Tester".into(), true.into()])
                    .values_panic([9.into(), "Reporter".into(), true.into()])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(RolesHierarchy::Table)
                    .columns([RolesHierarchy::RoleId, RolesHierarchy::ParentId])
                    .values_panic([2.into(), 1.into()])
                    .values_panic([3.into(), 2.into()])
                    .values_panic([4.into(), 3.into()])
                    .values_panic([5.into(), 4.into()])
                    .values_panic([6.into(), 5.into()])
                    .values_panic([7.into(), 4.into()])
                    .values_panic([8.into(), 4.into()])
                    .values_panic([9.into(), 4.into()])
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .truncate_table(Table::truncate().table(RolesHierarchy::Table).to_owned())
            .await
    }
}
