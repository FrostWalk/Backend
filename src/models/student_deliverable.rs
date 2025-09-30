use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use serde::Serialize;
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, Serialize, ToSchema, WeldsModel)]
#[welds(schema = "public", table = "student_deliverables")]
#[welds(HasMany(
    student_deliverables_components,
    StudentDeliverablesComponent,
    "student_deliverable_id"
))]
pub struct StudentDeliverable {
    #[welds(primary_key)]
    pub student_deliverable_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub name: String,
}
