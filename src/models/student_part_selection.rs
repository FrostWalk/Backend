use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_part_selections")]
pub struct StudentPartSelection {
    #[welds(primary_key)]
    pub student_part_selection_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "student_parts.student_part_id")]
    pub student_part_id: i32,
}
