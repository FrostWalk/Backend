use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "admins")]
pub struct Admin {
    #[welds(primary_key)]
    pub admin_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    #[welds(foreign_key = "admin_roles.admin_role_id")]
    pub admin_role_id: i32,
}
