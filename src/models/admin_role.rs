use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel, Serialize, Deserialize, ToSchema)]
#[welds(schema = "public", table = "admin_roles")]
pub struct AdminRole {
    #[welds(primary_key)]
    pub admin_role_id: i32,
    pub name: String,
}
