use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_deliverable_selections")]
pub struct StudentDeliverableSelection {
    #[welds(primary_key)]
    pub student_deliverable_selection_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "student_deliverables.student_deliverable_id")]
    pub student_deliverable_id: i32,
}
