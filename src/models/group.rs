use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "groups")]
pub struct Group {
    #[welds(primary_key)]
    pub group_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
