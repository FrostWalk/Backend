use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "security_codes")]
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
