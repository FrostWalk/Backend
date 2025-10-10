use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_deliverables_components")]
#[welds(BelongsTo(student_deliverable, StudentDeliverable, "student_deliverable_id"))]
#[welds(BelongsTo(
    student_deliverable_component,
    StudentDeliverableComponent,
    "student_deliverable_component_id"
))]
pub struct StudentDeliverablesComponent {
    #[welds(primary_key)]
    pub id: i32,
    #[welds(foreign_key = "student_deliverables.student_deliverable_id")]
    pub student_deliverable_id: i32,
    #[welds(foreign_key = "student_deliverable_components.student_deliverable_component_id")]
    pub student_deliverable_component_id: i32,
    pub quantity: i32,
}
