use chrono::{DateTime, Utc};
use welds::WeldsModel;

#[derive(Debug, Clone, WeldsModel)]
#[welds(schema = "public", table = "transactions")]
pub struct Transaction {
    #[welds(primary_key)]
    pub transaction_id: i32,
    #[welds(foreign_key = "groups.group_id")]
    pub buyer_group_id: i32,
    #[welds(foreign_key = "group_deliverable_selections.group_deliverable_selection_id")]
    pub group_deliverable_selection_id: i32,
    #[welds(foreign_key = "fairs.fair_id")]
    pub fair_id: i32,
    pub timestamp: DateTime<Utc>,
}
