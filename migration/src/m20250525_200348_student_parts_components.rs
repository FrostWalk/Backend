use crate::m20250525_200330_student_parts::StudentParts;
use crate::m20250525_200337_student_components::StudentsComponents;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentPartsComponents::Table)
                    .if_not_exists()
                    .col(integer(StudentPartsComponents::StudentPartId).not_null())
                    .col(integer(StudentPartsComponents::StudentsComponentId).not_null())
                    .col(integer(StudentPartsComponents::Quantity).not_null())
                    .primary_key(
                        Index::create()
                            .col(StudentPartsComponents::StudentPartId)
                            .col(StudentPartsComponents::StudentsComponentId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentPartsComponents::Table,
                                StudentPartsComponents::StudentPartId,
                            )
                            .to(StudentParts::Table, StudentParts::StudentPartId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentPartsComponents::Table,
                                StudentPartsComponents::StudentsComponentId,
                            )
                            .to(
                                StudentsComponents::Table,
                                StudentsComponents::StudentsComponentId,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StudentPartsComponents::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum StudentPartsComponents {
    Table,
    StudentPartId,
    StudentsComponentId,
    Quantity,
}
