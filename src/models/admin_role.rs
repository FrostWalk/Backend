use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "admin_roles")]
pub struct AdminRole {
    #[welds(primary_key)]
    pub admin_role_id: i32,
    pub name: String,
}
