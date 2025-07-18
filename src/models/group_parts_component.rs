use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "group_parts_components")]
pub struct GroupPartsComponent {
    #[welds(foreign_key = "group_components.group_component_id")]
    pub group_component_id: i32,
    #[welds(foreign_key = "group_parts.group_part_id")]
    pub group_part_id: i32,
    pub quantity: i32,
}
