use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Projects table
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).string().not_null())
                    .col(ColumnDef::new(Projects::Year).tiny_unsigned().not_null())
                    .col(
                        ColumnDef::new(Projects::MaxGroupSize)
                            .tiny_unsigned()
                            .not_null(),
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

        // Admins Table
        manager
            .create_table(
                Table::create()
                    .table(Admins::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Admins::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Admins::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Admins::Email).string().null().unique_key())
                    .col(ColumnDef::new(Admins::PasswordHash).blob().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Admins::Table)
                    .col(Admins::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Admins::Table)
                    .col(Admins::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Project Components table
        manager
            .create_table(
                Table::create()
                    .table(ProjectComponents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectComponents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectComponents::ProjectId)
                            .integer()
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

        // AuxiliaryRoles table
        manager
            .create_table(
                Table::create()
                    .table(AuxiliaryRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuxiliaryRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuxiliaryRoles::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuxiliaryRoles::Table)
                    .col(AuxiliaryRoles::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
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
                    .col(
                        ColumnDef::new(Users::StudentId)
                            .integer()
                            .null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).blob().not_null())
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
        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .col(Users::StudentId)
                    .unique()
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectOptions::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ProjectOptions::Table)
                    .col(ProjectOptions::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Settings
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Settings::Name).string().not_null())
                    .col(ColumnDef::new(Settings::Value).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ProjectOptions::Table)
                    .col(ProjectOptions::Name)
                    .unique()
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::OptionId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OptionsComponentsAndQuantity::ComponentId)
                            .integer()
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(ColumnDef::new(Groups::OptionId).integer().not_null())
                    .col(ColumnDef::new(Groups::ProjectId).integer().not_null())
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

        // Groups Project Components table
        manager
            .create_table(
                Table::create()
                    .table(GroupsAndProjectComponents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::ComponentId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::CustomName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GroupsAndProjectComponents::MarkDownDescription)
                            .text()
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

        // Fair table
        manager
            .create_table(
                Table::create()
                    .table(Fairs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Fairs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Fairs::StartDate).date_time().null())
                    .col(ColumnDef::new(Fairs::EndDate).date_time().null())
                    .col(ColumnDef::new(Fairs::ProjectId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Fairs::Table, Fairs::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Fairs purchase table
        manager
            .create_table(
                Table::create()
                    .table(FairsPurchasing::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FairsPurchasing::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FairsPurchasing::DateOfPurchase)
                            .date_time()
                            .null(),
                    )
                    .col(ColumnDef::new(FairsPurchasing::FairId).integer().not_null())
                    .col(
                        ColumnDef::new(FairsPurchasing::PurchasedComponentId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FairsPurchasing::PurchasingGroupId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FairsPurchasing::Table, FairsPurchasing::FairId)
                            .to(Fairs::Table, Fairs::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FairsPurchasing::Table, FairsPurchasing::PurchasingGroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                FairsPurchasing::Table,
                                FairsPurchasing::PurchasedComponentId,
                            )
                            .to(
                                GroupsAndProjectComponents::Table,
                                GroupsAndProjectComponents::Id,
                            )
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(FairsPurchasing::Table)
                    .col(FairsPurchasing::PurchasingGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(FairsPurchasing::Table)
                    .col(FairsPurchasing::PurchasedComponentId)
                    .to_owned(),
            )
            .await?;

        // Students Groups table
        manager
            .create_table(
                Table::create()
                    .table(StudentsAndGroups::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentsAndGroups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndGroups::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndGroups::UserId)
                            .integer()
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

        // Users Projects Roles table
        manager
            .create_table(
                Table::create()
                    .table(UsersProjectsAndRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::RoleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::AuxiliaryRoleId)
                            .null()
                            .integer(),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::HasRetired)
                            .not_null()
                            .boolean()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UsersProjectsAndRoles::RetirementDate)
                            .null()
                            .date_time(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersProjectsAndRoles::Table, UsersProjectsAndRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                UsersProjectsAndRoles::Table,
                                UsersProjectsAndRoles::ProjectId,
                            )
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersProjectsAndRoles::Table, UsersProjectsAndRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                UsersProjectsAndRoles::Table,
                                UsersProjectsAndRoles::AuxiliaryRoleId,
                            )
                            .to(AuxiliaryRoles::Table, AuxiliaryRoles::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(UsersProjectsAndRoles::Table)
                    .col(UsersProjectsAndRoles::UserId)
                    .col(UsersProjectsAndRoles::ProjectId)
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SecurityCodes::GroupId).integer().not_null())
                    .col(
                        ColumnDef::new(SecurityCodes::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SecurityCodes::SecurityCodeHash)
                            .binary()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SecurityCodes::RoleId).integer().not_null())
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(IndividualWorkOptions::ProjectId)
                            .integer()
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
                    .table(StudentsAndIndividualWork::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::IndividualWorkOptionId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::FileName)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::FileHash)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(StudentsAndIndividualWork::DateOfUpload)
                            .date()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentsAndIndividualWork::Table,
                                StudentsAndIndividualWork::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                StudentsAndIndividualWork::Table,
                                StudentsAndIndividualWork::IndividualWorkOptionId,
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
                    .table(StudentsAndIndividualWork::Table)
                    .col(StudentsAndIndividualWork::UserId)
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Complaints::FromGroupId).integer().not_null())
                    .col(ColumnDef::new(Complaints::ToGroupId).integer().not_null())
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
            .await?;

        // Blacklist table
        manager
            .create_table(
                Table::create()
                    .table(BlackList::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BlackList::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BlackList::UserId).integer().not_null())
                    .col(ColumnDef::new(BlackList::NoteText).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(BlackList::Table, BlackList::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(BlackList::Table)
                    .col(BlackList::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Admins::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(BlackList::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Complaints::Table).to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(StudentsAndIndividualWork::Table)
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
            .drop_table(Table::drop().table(UsersProjectsAndRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StudentsAndGroups::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(FairsPurchasing::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Fairs::Table).to_owned())
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
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuxiliaryRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectComponents::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Projects {
    Table,
    Id,
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
pub enum Settings {
    Table,
    Id,
    Name,
    Value,
}

#[derive(Iden)]
pub enum Admins {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Name,
    Surname,
    Email,
    StudentId,
    PasswordHash,
    TelegramNick,
}

#[derive(Iden)]
pub enum Roles {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
pub enum AuxiliaryRoles {
    Table,
    Id,
    Name,
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
    CodeLink,
    MarkDownDescription,
    TelegramSupportLink,
}

#[derive(Iden)]
pub enum Fairs {
    Table,
    Id,
    ProjectId,
    StartDate,
    EndDate,
}

#[derive(Iden)]
pub enum FairsPurchasing {
    Table,
    Id,
    FairId,
    PurchasedComponentId,
    PurchasingGroupId,
    DateOfPurchase,
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
pub enum UsersProjectsAndRoles {
    Table,
    Id,
    UserId,
    ProjectId,
    RoleId,
    AuxiliaryRoleId,
    HasRetired,
    RetirementDate,
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
pub enum StudentsAndIndividualWork {
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

#[derive(Iden)]
pub enum BlackList {
    Table,
    Id,
    UserId,
    NoteText,
}
