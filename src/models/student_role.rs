use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "student_roles")]
pub struct StudentRole {
    #[welds(primary_key)]
    pub student_role_id: i32,
    pub name: String,
}
