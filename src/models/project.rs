use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel, Serialize, Deserialize, ToSchema)]
#[welds(schema = "public", table = "projects")]
#[welds(HasMany(group_deliverables, GroupDeliverable, "project_id"))]
#[welds(HasMany(group_deliverable_components, GroupDeliverableComponent, "project_id"))]
#[welds(HasMany(student_deliverables, StudentDeliverable, "project_id"))]
#[welds(HasMany(
    student_deliverable_components,
    StudentDeliverableComponent,
    "project_id"
))]
pub struct Project {
    #[welds(primary_key)]
    pub project_id: i32,
    pub name: String,
    pub year: i32,
    pub max_student_uploads: i32,
    pub max_group_size: i32,
    pub active: bool,
}
