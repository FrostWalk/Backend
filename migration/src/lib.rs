pub use sea_orm_migration::prelude::*;
mod m20250524_184046_projects;
mod m20250524_184935_admin_roles;
mod m20250524_185005_admins;
mod m20250524_212554_students_roles;
mod m20250524_212557_students;
mod m20250524_212919_security_codes;
mod m20250524_213908_blacklist;
mod m20250524_222426_groups;
mod m20250524_223708_group_members;
mod m20250524_224854_complaints;
mod m20250524_225515_fairs;
mod m20250524_232013_group_parts;
mod m20250524_232024_group_components;
mod m20250524_232040_group_parts_components;
mod m20250524_232050_group_part_selections;
mod m20250525_194150_transactions;
mod m20250525_200330_student_parts;
mod m20250525_200337_student_components;
mod m20250525_200348_student_parts_components;
mod m20250525_200353_student_part_selections;
mod m20250525_200401_student_uploads;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250524_184046_projects::Migration),
            Box::new(m20250524_184935_admin_roles::Migration),
            Box::new(m20250524_185005_admins::Migration),
            Box::new(m20250524_212554_students_roles::Migration),
            Box::new(m20250524_212557_students::Migration),
            Box::new(m20250524_212919_security_codes::Migration),
            Box::new(m20250524_213908_blacklist::Migration),
            Box::new(m20250524_222426_groups::Migration),
            Box::new(m20250524_223708_group_members::Migration),
            Box::new(m20250524_224854_complaints::Migration),
            Box::new(m20250524_225515_fairs::Migration),
            Box::new(m20250524_232013_group_parts::Migration),
            Box::new(m20250524_232024_group_components::Migration),
            Box::new(m20250524_232040_group_parts_components::Migration),
            Box::new(m20250524_232050_group_part_selections::Migration),
            Box::new(m20250525_194150_transactions::Migration),
            Box::new(m20250525_200330_student_parts::Migration),
            Box::new(m20250525_200337_student_components::Migration),
            Box::new(m20250525_200348_student_parts_components::Migration),
            Box::new(m20250525_200353_student_part_selections::Migration),
            Box::new(m20250525_200401_student_uploads::Migration),
        ]
    }
}
