use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_deliverables_components")]
pub struct GroupDeliverablesComponent {
    #[welds(foreign_key = "group_deliverables.group_deliverable_id")]
    pub group_deliverable_id: i32,
    #[welds(foreign_key = "group_deliverable_components.group_deliverable_component_id")]
    pub group_deliverable_component_id: i32,
    pub quantity: i32,
}
