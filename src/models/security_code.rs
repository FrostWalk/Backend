use crate::models::project::Project;
use crate::models::student_role::StudentRole;
use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "security_codes")]
#[welds(BelongsTo(project, Project, "project_id"))]
#[welds(BelongsTo(student_role, StudentRole, "student_role_id"))]
pub struct SecurityCode {
    #[welds(primary_key)]
    pub security_code_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    #[welds(foreign_key = "student_roles.student_role_id")]
    pub student_role_id: i32,
    pub code: String,
    pub expiration: DateTime<Utc>,
}