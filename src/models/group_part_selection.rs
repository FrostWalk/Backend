use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "group_part_selections")]
pub struct GroupPartSelection {
    #[welds(primary_key)]
    pub group_part_selection_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub group_id: i32,
    pub link: String,
    pub markdown_text: String,
}
