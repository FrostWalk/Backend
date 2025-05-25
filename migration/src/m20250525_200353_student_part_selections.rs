use crate::m20250524_212557_students::Students;
use crate::m20250525_200330_student_parts::StudentParts;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentPartSelections::Table)
                    .if_not_exists()
                    .col(pk_auto(StudentPartSelections::StudentPartSelectionId))
                    .col(integer(StudentPartSelections::StudentId).not_null())
                    .col(integer(StudentPartSelections::StudentPartId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentPartSelections::Table,
                                StudentPartSelections::StudentId,
                            )
                            .to(Students::Table, Students::StudentId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentPartSelections::Table,
                                StudentPartSelections::StudentPartId,
                            )
                            .to(StudentParts::Table, StudentParts::StudentPartId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentPartSelections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum StudentPartSelections {
    Table,
    StudentPartSelectionId,
    StudentId,
    StudentPartId,
}
