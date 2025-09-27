use num_enum::{IntoPrimitive, TryFromPrimitive};
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

#[derive(PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub(crate) enum AvailableAdminRole {
    Root = 1,
    Professor = 2,
    Tutor = 3,
    Coordinator = 4,
}
pub(crate) const ALL: [AvailableAdminRole; 4] = [
    AvailableAdminRole::Root,
    AvailableAdminRole::Professor,
    AvailableAdminRole::Tutor,
    AvailableAdminRole::Coordinator,
];
