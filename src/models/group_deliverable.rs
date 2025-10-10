use crate::models::group_deliverables_component::GroupDeliverablesComponent;
use serde::Serialize;
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, Serialize, ToSchema, WeldsModel)]
#[welds(schema = "public", table = "group_deliverables")]
#[welds(HasMany(
    group_deliverables_components,
    GroupDeliverablesComponent,
    "group_deliverable_id"
))]
pub struct GroupDeliverable {
    #[welds(primary_key)]
    pub group_deliverable_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
