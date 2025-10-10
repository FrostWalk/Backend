use crate::models::student::Student;
use crate::models::student_deliverable::StudentDeliverable;
use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_deliverable_selections")]
#[welds(BelongsTo(student, Student, "student_id"))]
#[welds(BelongsTo(student_deliverable, StudentDeliverable, "student_deliverable_id"))]
pub struct StudentDeliverableSelection {
    #[welds(primary_key)]
    pub student_deliverable_selection_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "student_deliverables.student_deliverable_id")]
    pub student_deliverable_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
