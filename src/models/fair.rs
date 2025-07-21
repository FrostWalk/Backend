use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "fairs")]
pub struct Fair {
    #[welds(primary_key)]
    pub fair_id: i32,
    #[welds(foreign_key = "projects.project_id")]
    pub project_id: i32,
    pub details: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}
