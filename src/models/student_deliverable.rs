use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_deliverables")]
pub struct StudentDeliverable {
    #[welds(primary_key)]
    pub student_deliverable_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
