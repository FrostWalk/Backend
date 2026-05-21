use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel, Serialize, Deserialize, ToSchema)]
#[welds(schema = "public", table = "oral_exam_completions")]
pub struct OralExamCompletion {
    #[welds(primary_key)]
    pub completion_id: i32,
    #[welds(foreign_key = "students.student_id")]
    pub student_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub completed_at: DateTime<Utc>,
    pub completed_by_admin_id: Option<i32>,
}
