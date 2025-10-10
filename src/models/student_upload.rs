use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "student_uploads")]
pub struct StudentUpload {
    #[welds(primary_key)]
    pub upload_id: i32,
    #[welds(foreign_key = "student_deliverable_selections.student_deliverable_selection_id")]
    pub student_deliverable_selection_id: i32,
    pub path: String,
    pub timestamp: DateTime<Utc>,
}
