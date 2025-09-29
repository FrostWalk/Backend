use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_deliverable_components")]
pub struct StudentDeliverableComponent {
    #[welds(primary_key)]
    pub student_deliverable_component_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
