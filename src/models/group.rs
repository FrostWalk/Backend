use crate::models::project::Project;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "groups")]
#[welds(BelongsTo(project, Project, "project_id"))]
pub struct Group {
    #[welds(primary_key)]
    pub group_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
