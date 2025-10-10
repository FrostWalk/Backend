use crate::models::admin::Admin;
use crate::models::project::Project;
use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "coordinator_projects")]
#[welds(BelongsTo(admin, Admin, "admin_id"))]
#[welds(BelongsTo(project, Project, "project_id"))]
pub struct CoordinatorProject {
    #[welds(primary_key)]
    pub coordinator_project_id: i32,
    #[welds(foreign_key = "admins.admin_id")]
    pub admin_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub assigned_at: DateTime<Utc>,
}
