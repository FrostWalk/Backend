use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_parts_components")]
pub struct StudentPartsComponent {
    #[welds(foreign_key = "student_parts.student_part_id")]
    pub student_part_id: i32,
    #[welds(foreign_key = "students_components.students_component_id")]
    pub students_component_id: i32,
    pub quantity: i32,
}
