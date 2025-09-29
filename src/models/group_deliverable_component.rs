use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_deliverable_components")]
pub struct GroupDeliverableComponent {
    #[welds(primary_key)]
    pub group_deliverable_component_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
