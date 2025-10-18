use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::group_deliverable_selection::GroupDeliverableSelection;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, Serialize, ToSchema, WeldsModel)]
#[welds(schema = "public", table = "group_component_implementation_details")]
#[welds(BelongsTo(
    group_deliverable_selection,
    GroupDeliverableSelection,
    "group_deliverable_selection_id"
))]
#[welds(BelongsTo(
    group_deliverable_component,
    GroupDeliverableComponent,
    "group_deliverable_component_id"
))]
pub struct GroupComponentImplementationDetail {
    #[welds(primary_key)]
    pub id: i32,
    #[welds(foreign_key = "group_deliverable_selections.group_deliverable_selection_id")]
    pub group_deliverable_selection_id: i32,
    #[welds(foreign_key = "group_deliverable_components.group_deliverable_component_id")]
    pub group_deliverable_component_id: i32,
    pub markdown_description: String,
    pub repository_link: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
