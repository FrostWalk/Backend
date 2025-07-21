use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_members")]
pub struct GroupMember {
    #[welds(primary_key)]
    pub group_member_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub group_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "student_roles.student_role_id")]
    pub student_role_id: i32,
}
