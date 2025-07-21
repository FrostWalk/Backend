use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "projects")]
pub struct Project {
    #[welds(primary_key)]
    pub project_id: i32,
    pub name: String,
    pub year: i32,
    pub max_student_uploads: i32,
    pub max_group_size: i32,
    pub active: bool,
}
