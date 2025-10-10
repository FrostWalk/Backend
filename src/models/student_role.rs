use num_enum::{IntoPrimitive, TryFromPrimitive};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_roles")]
pub struct StudentRole {
    #[welds(primary_key)]
    pub student_role_id: i32,
    pub name: String,
}

#[derive(PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub(crate) enum AvailableStudentRole {
    GroupLeader = 1,
    Member = 2,
}
