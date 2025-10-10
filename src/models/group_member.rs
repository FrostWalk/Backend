use crate::models::group::Group;
use crate::models::student::Student;
use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_members")]
#[welds(BelongsTo(group, Group, "group_id"))]
#[welds(BelongsTo(student, Student, "student_id"))]
pub struct GroupMember {
    #[welds(primary_key)]
    pub group_member_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub group_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "student_roles.student_role_id")]
    pub student_role_id: i32,
    pub joined_at: DateTime<Utc>,
}
