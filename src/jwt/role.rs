use crate::jwt::role::UserRole::{
    GroupLeader, GroupMember, Professor, Tutor, WorkingGroupCoordinator,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum UserRole {
    Professor = 1,
    Tutor = 2,
    WorkingGroupCoordinator = 3,
    GroupLeader = 4,
    GroupMember = 5,
}

impl Into<UserRole> for i32 {
    fn into(self) -> UserRole {
        match self {
            1 => Professor,
            2 => Tutor,
            3 => WorkingGroupCoordinator,
            4 => GroupLeader,
            5 => GroupMember,
            _ => unreachable!(),
        }
    }
}

pub(crate) const ALL: [UserRole; 5] = [
    Professor,
    Tutor,
    WorkingGroupCoordinator,
    GroupLeader,
    GroupMember,
];
