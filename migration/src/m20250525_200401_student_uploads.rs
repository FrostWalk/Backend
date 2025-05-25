use crate::m20250525_200353_student_part_selections::StudentPartSelections;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentUploads::Table)
                    .if_not_exists()
                    .col(pk_auto(StudentUploads::UploadId))
                    .col(integer(StudentUploads::StudentPartSelectionId).not_null())
                    .col(string(StudentUploads::Path).not_null().unique_key())
                    .col(timestamp(StudentUploads::Timestamp).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentUploads::Table,
                                StudentUploads::StudentPartSelectionId,
                            )
                            .to(
                                StudentPartSelections::Table,
                                StudentPartSelections::StudentPartSelectionId,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentUploads::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum StudentUploads {
    Table,
    UploadId,
    StudentPartSelectionId,
    Path,
    Timestamp,
}
