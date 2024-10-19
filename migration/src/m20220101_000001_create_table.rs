use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Courses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Courses::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Courses::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::CourseId).big_integer().not_null())
                    .col(ColumnDef::new(Projects::Name).string().not_null())
                    .col(ColumnDef::new(Projects::Year).tiny_unsigned().not_null())
                    .col(
                        ColumnDef::new(Projects::MaxGroupSize)
                            .tiny_unsigned()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Projects::Table, Projects::CourseId)
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Projects::Table)
                    .col(Projects::Name)
                    .col(Projects::Year)
                    .col(Projects::CourseId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Projects::Table)
                    .col((Projects::Year, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Projects::Table)
                    .col(Projects::CourseId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectComponents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectComponents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectComponents::ProjectId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectComponents::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectComponents::Table, ProjectComponents::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ProjectComponents::Table)
                    .col(ProjectComponents::Name)
                    .col(ProjectComponents::ProjectId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::Surname).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).blob().not_null())
                    .col(ColumnDef::new(Users::Salt).blob().not_null())
                    .col(ColumnDef::new(Users::TelegramNick).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .col(Users::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .col(Users::Surname)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Roles table
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Roles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Roles::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Roles::IsAuxiliary).boolean().default(true))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Roles::Table)
                    .col(Roles::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // RolesHierarchy table
        manager
            .create_table(
                Table::create()
                    .table(RolesHierarchy::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolesHierarchy::RoleId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(RolesHierarchy::ParentId)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RolesHierarchy::Table, RolesHierarchy::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .to(Roles::Table, Roles::Id)
                            .from(RolesHierarchy::Table, RolesHierarchy::ParentId)
                            .on_delete(ForeignKeyAction::SetDefault),
                    )
                    .primary_key(
                        Index::create()
                            .col(RolesHierarchy::RoleId)
                            .col(RolesHierarchy::ParentId),
                    )
                    .to_owned(),
            )
            .await?;

        // ProjectOptions table
        manager
            .create_table(
                Table::create()
                    .table(ProjectOptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectOptions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectOptions::Name).string().null())
                    .to_owned(),
            )
            .await?;

        // OptionsComponentsAndQuantity table
        manager
            .create_table(
                Table::create()
                    .table(OptionsComponentsAndQuantity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::OptionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::ComponentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::Quantity)
                            .tiny_unsigned()
                            .not_null()
                            .default(1),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                OptionsComponentsAndQuantity::Table,
                                OptionsComponentsAndQuantity::OptionId,
                            )
                            .to(ProjectOptions::Table, ProjectOptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                OptionsComponentsAndQuantity::Table,
                                OptionsComponentsAndQuantity::ComponentId,
                            )
                            .to(ProjectComponents::Table, ProjectComponents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(OptionsComponentsAndQuantity::Table)
                    .col(OptionsComponentsAndQuantity::OptionId)
                    .col(OptionsComponentsAndQuantity::ComponentId)
                    .to_owned(),
            )
            .await?;

        // Groups table
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Groups::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(ColumnDef::new(Groups::OptionId).big_integer().not_null())
                    .col(ColumnDef::new(Groups::ProjectId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Groups::Table, Groups::OptionId)
                            .to(ProjectOptions::Table, ProjectOptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Groups::Table, Groups::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Groups::Table)
                    .col(Groups::Name)
                    .col(Groups::ProjectId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupsAndProjectComponents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::GroupId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::ComponentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::CustomName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::FlyerName)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::CodeLink)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::TelegramSupportLink)
                            .text()
                            .null()
                            .unique_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                GroupsAndProjectComponents::Table,
                                GroupsAndProjectComponents::GroupId,
                            )
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                GroupsAndProjectComponents::Table,
                                GroupsAndProjectComponents::ComponentId,
                            )
                            .to(ProjectComponents::Table, ProjectComponents::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StudentsAndGroups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentsAndGroups::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndGroups::GroupId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndGroups::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndGroups::IsRetired)
                            .boolean()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StudentsAndGroups::Table, StudentsAndGroups::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(StudentsAndGroups::Table, StudentsAndGroups::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(StudentsAndGroups::Table)
                    .col(StudentsAndGroups::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UsersProjectsRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersProjectsRoles::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsRoles::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsRoles::ProjectId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersProjectsRoles::Table, UsersProjectsRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersProjectsRoles::Table, UsersProjectsRoles::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersProjectsRoles::Table, UsersProjectsRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(UsersProjectsRoles::Table)
                    .col(UsersProjectsRoles::UserId)
                    .col(UsersProjectsRoles::ProjectId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // SecurityCodes table
        manager
            .create_table(
                Table::create()
                    .table(SecurityCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SecurityCodes::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SecurityCodes::GroupId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SecurityCodes::ProjectId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SecurityCodes::SecurityCodeHash)
                            .binary()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SecurityCodes::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SecurityCodes::ValidUntil).date().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(SecurityCodes::Table, SecurityCodes::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SecurityCodes::Table, SecurityCodes::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SecurityCodes::Table, SecurityCodes::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(SecurityCodes::Table)
                    .col(SecurityCodes::SecurityCodeHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // IndividualWorkOptions table
        manager
            .create_table(
                Table::create()
                    .table(IndividualWorkOptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IndividualWorkOptions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(IndividualWorkOptions::ProjectId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IndividualWorkOptions::Name)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                IndividualWorkOptions::Table,
                                IndividualWorkOptions::ProjectId,
                            )
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(IndividualWorkOptions::Table)
                    .col(IndividualWorkOptions::ProjectId)
                    .to_owned(),
            )
            .await?;

        // StudentsIndividualWork table
        manager
            .create_table(
                Table::create()
                    .table(StudentsIndividualWork::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentsIndividualWork::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentsIndividualWork::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsIndividualWork::IndividualWorkOptionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsIndividualWork::FileName)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StudentsIndividualWork::FileHash)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StudentsIndividualWork::DateOfUpload)
                            .date()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentsIndividualWork::Table,
                                StudentsIndividualWork::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentsIndividualWork::Table,
                                StudentsIndividualWork::IndividualWorkOptionId,
                            )
                            .to(IndividualWorkOptions::Table, IndividualWorkOptions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(StudentsIndividualWork::Table)
                    .col(StudentsIndividualWork::UserId)
                    .to_owned(),
            )
            .await?;

        // Complaints table
        manager
            .create_table(
                Table::create()
                    .table(Complaints::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Complaints::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Complaints::FromGroupId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Complaints::ToGroupId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Complaints::ComplainText).text().not_null())
                    .col(ColumnDef::new(Complaints::DateOfCreation).date().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Complaints::Table, Complaints::FromGroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Complaints::Table, Complaints::ToGroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Complaints::Table)
                    .col(Complaints::ToGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Complaints::Table)
                    .col(Complaints::FromGroupId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(Complaints::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(StudentsIndividualWork::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(IndividualWorkOptions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SecurityCodes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UsersProjectsRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StudentsAndGroups::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(GroupsAndProjectComponents::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Groups::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(OptionsComponentsAndQuantity::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectOptions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolesHierarchy::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectComponents::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Courses::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Courses {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
pub enum Projects {
    Table,
    Id,
    CourseId,
    Name,
    Year,
    MaxGroupSize,
}

#[derive(Iden)]
pub enum ProjectComponents {
    Table,
    Id,
    ProjectId,
    Name,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Name,
    Surname,
    Email,
    PasswordHash,
    Salt,
    TelegramNick,
}

#[derive(Iden)]
pub enum Roles {
    Table,
    Id,
    Name,
    IsAuxiliary,
}

#[derive(Iden)]
pub enum RolesHierarchy {
    Table,
    RoleId,
    ParentId,
}

#[derive(Iden)]
pub enum ProjectOptions {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
pub enum OptionsComponentsAndQuantity {
    Table,
    Id,
    OptionId,
    ComponentId,
    Quantity,
}

#[derive(Iden)]
pub enum Groups {
    Table,
    Id,
    Name,
    OptionId,
    ProjectId,
}

#[derive(Iden)]
pub enum GroupsAndProjectComponents {
    Table,
    Id,
    GroupId,
    ComponentId,
    CustomName,
    FlyerName,
    CodeLink,
    TelegramSupportLink,
}

#[derive(Iden)]
pub enum StudentsAndGroups {
    Table,
    Id,
    GroupId,
    UserId,
    IsRetired,
}

#[derive(Iden)]
pub enum UsersProjectsRoles {
    Table,
    Id,
    UserId,
    ProjectId,
    RoleId,
}

#[derive(Iden)]
pub enum SecurityCodes {
    Table,
    Id,
    GroupId,
    ProjectId,
    SecurityCodeHash,
    RoleId,
    ValidUntil,
}

#[derive(Iden)]
pub enum IndividualWorkOptions {
    Table,
    Id,
    ProjectId,
    Name,
}

#[derive(Iden)]
pub enum StudentsIndividualWork {
    Table,
    Id,
    UserId,
    IndividualWorkOptionId,
    FileName,
    FileHash,
    DateOfUpload,
}

#[derive(Iden)]
pub enum Complaints {
    Table,
    Id,
    FromGroupId,
    ToGroupId,
    ComplainText,
    DateOfCreation,
}
