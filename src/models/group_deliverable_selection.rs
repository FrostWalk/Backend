use crate::models::group::Group;
use crate::models::group_deliverable::GroupDeliverable;
use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_deliverable_selections")]
#[welds(BelongsTo(group, Group, "group_id"))]
#[welds(BelongsTo(group_deliverable, GroupDeliverable, "group_deliverable_id"))]
pub struct GroupDeliverableSelection {
    #[welds(primary_key)]
    pub group_deliverable_selection_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub group_id: i32,
    #[welds(foreign_key = "group_deliverables.group_deliverable_id")]
    pub group_deliverable_id: i32,
    pub link: String,
    pub markdown_text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
