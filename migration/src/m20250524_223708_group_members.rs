use crate::m20250524_212554_students_roles::StudentRoles;
use crate::m20250524_212557_students::Students;
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
                    .table(GroupMembers::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupMembers::GroupMemberId))
                    .col(integer(GroupMembers::GroupId).not_null())
                    .col(integer(GroupMembers::StudentId).not_null())
                    .col(integer(GroupMembers::StudentRoleId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMembers::Table, GroupMembers::GroupId)
                            .to(Groups::Table, GroupMembers::GroupId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMembers::Table, GroupMembers::StudentId)
                            .to(Students::Table, Students::StudentId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMembers::Table, GroupMembers::StudentRoleId)
                            .to(StudentRoles::Table, StudentRoles::StudentRoleId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .if_not_exists()
                            .table(GroupMembers::Table)
                            .col(GroupMembers::GroupId)
                            .col(GroupMembers::StudentId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GroupMembers {
    Table,
    GroupMemberId,
    GroupId,
    StudentId,
    StudentRoleId,
}
