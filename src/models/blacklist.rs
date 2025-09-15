use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel, Serialize, Deserialize, ToSchema)]
#[welds(schema = "public", table = "blacklist")]
pub struct Blacklist {
    #[welds(primary_key)]
    pub blacklist_id: i32,
    pub university_id: i32,
    pub description: String,
    pub first_name: String,
    pub last_name: String,
    pub banned_at: DateTime<Utc>,
}
