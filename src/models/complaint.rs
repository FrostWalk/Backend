use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "complaints")]
pub struct Complaint {
    #[welds(primary_key)]
    pub complaint_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub from_group_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub to_group_id: i32,
    pub text: String,
    pub created_at: DateTime<Utc>,
}
