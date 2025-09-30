use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_deliverables_components")]
#[welds(BelongsTo(group_deliverable, GroupDeliverable, "group_deliverable_id"))]
#[welds(BelongsTo(
    group_deliverable_component,
    GroupDeliverableComponent,
    "group_deliverable_component_id"
))]
pub struct GroupDeliverablesComponent {
    #[welds(primary_key)]
    pub id: i32,
    #[welds(foreign_key = "group_deliverables.group_deliverable_id")]
    pub group_deliverable_id: i32,
    #[welds(foreign_key = "group_deliverable_components.group_deliverable_component_id")]
    pub group_deliverable_component_id: i32,
    pub quantity: i32,
}
