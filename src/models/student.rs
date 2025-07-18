use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "students")]
pub struct Student {
    #[welds(primary_key)]
    pub student_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub university_id: i32,
    pub password_hash: String,
    pub is_pending: bool,
}
