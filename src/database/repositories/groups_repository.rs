use crate::models::group::Group;
use crate::models::group_member::GroupMember;
use crate::models::project::Project;
use crate::models::student_role::AvailableStudentRole;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Create a new group
pub(crate) async fn create_group(
    db: &PostgresClient, group: Group,
) -> welds::errors::Result<DbState<Group>> {
    let mut state = DbState::new_uncreated(group);
    state.save(db).await?;
    Ok(state)
}

/// Create a new group member
pub(crate) async fn create_group_member(
    db: &PostgresClient, group_member: GroupMember,
) -> welds::errors::Result<DbState<GroupMember>> {
    let mut state = DbState::new_uncreated(group_member);
    state.save(db).await?;
    Ok(state)
}

/// Get a group by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Option<DbState<Group>>> {
    let mut rows = Group::where_col(|g| g.group_id.equal(group_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Get all groups for a specific project
pub(crate) async fn get_by_project_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Vec<DbState<Group>>> {
    Group::where_col(|g| g.project_id.equal(project_id))
        .run(db)
        .await
}

/// Get all members of a group
pub(crate) async fn get_members(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupMember>>> {
    GroupMember::where_col(|gm| gm.group_id.equal(group_id))
        .run(db)
        .await
}

/// Get all members of a group (alias for get_members)
pub(crate) async fn get_group_members(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<Vec<DbState<GroupMember>>> {
    get_members(db, group_id).await
}

/// Count the number of members in a group
pub(crate) async fn count_members(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<i32> {
    let members = get_members(db, group_id).await?;
    Ok(members.len() as i32)
}

/// Check if a student is a group leader of a specific group
pub(crate) async fn is_group_leader(
    db: &PostgresClient, student_id: i32, group_id: i32,
) -> welds::errors::Result<bool> {
    let members = get_members(db, group_id).await?;

    for member_state in members {
        let member = DbState::into_inner(member_state);
        if member.student_id == student_id
            && member.student_role_id == AvailableStudentRole::GroupLeader as i32
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if a student is in any group for a specific project
pub(crate) async fn is_student_in_project(
    db: &PostgresClient, student_id: i32, project_id: i32,
) -> welds::errors::Result<bool> {
    let existing_membership = GroupMember::where_col(|gm| gm.student_id.equal(student_id))
        .run(db)
        .await?;

    for membership in existing_membership {
        let membership_data = DbState::into_inner(membership);
        let group_states = Group::where_col(|g| g.group_id.equal(membership_data.group_id))
            .run(db)
            .await
            .unwrap_or_default();

        if let Some(group_state) = group_states.into_iter().next() {
            let group = DbState::into_inner(group_state);
            if group.project_id == project_id {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Delete a group and all its members
pub(crate) async fn delete_group_with_members(
    db: &PostgresClient, group_id: i32,
) -> welds::errors::Result<()> {
    // Delete all group members first
    GroupMember::where_col(|gm| gm.group_id.equal(group_id))
        .delete(db)
        .await?;

    // Delete the group
    Group::where_col(|g| g.group_id.equal(group_id))
        .delete(db)
        .await?;

    Ok(())
}

/// Check if a group name already exists for a project
pub(crate) async fn name_exists_for_project(
    db: &PostgresClient, project_id: i32, name: &str,
) -> welds::errors::Result<bool> {
    let rows = Group::where_col(|g| g.project_id.equal(project_id))
        .run(db)
        .await?;

    for group_state in rows {
        let group = DbState::into_inner(group_state);
        if group.name == name {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Count the number of groups for a specific project
pub(crate) async fn count_groups_for_project(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<i32> {
    let groups = get_by_project_id(db, project_id).await?;
    Ok(groups.len() as i32)
}

/// Delete a group by ID
pub(crate) async fn delete_by_id(db: &PostgresClient, group_id: i32) -> welds::errors::Result<()> {
    Group::where_col(|g| g.group_id.equal(group_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Delete a group member by ID
pub(crate) async fn delete_member_by_id(
    db: &PostgresClient, group_member_id: i32,
) -> welds::errors::Result<()> {
    GroupMember::where_col(|gm| gm.group_member_id.equal(group_member_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Get all groups for a student with their projects (returns GroupMember -> Group -> Project tuples)
pub(crate) async fn get_groups_with_projects_for_student(
    db: &PostgresClient, student_id: i32,
) -> welds::errors::Result<Vec<(DbState<GroupMember>, DbState<Group>, DbState<Project>)>> {
    let group_members = GroupMember::where_col(|gm| gm.student_id.equal(student_id))
        .run(db)
        .await?;

    let mut result = Vec::new();
    for gm in group_members {
        let mut groups = Group::where_col(|g| g.group_id.equal(gm.group_id))
            .run(db)
            .await?;

        if let Some(group) = groups.pop() {
            let mut projects = Project::where_col(|p| p.project_id.equal(group.project_id))
                .run(db)
                .await?;

            if let Some(project) = projects.pop() {
                result.push((gm, group, project));
            }
        }
    }

    Ok(result)
}

/// Get all groups for a student (returns GroupMember -> Group tuples)
pub(crate) async fn get_groups_for_student(
    db: &PostgresClient, student_id: i32,
) -> welds::errors::Result<Vec<(DbState<GroupMember>, DbState<Group>)>> {
    let group_members = GroupMember::where_col(|gm| gm.student_id.equal(student_id))
        .run(db)
        .await?;

    let mut result = Vec::new();
    for gm in group_members {
        let mut groups = Group::where_col(|g| g.group_id.equal(gm.group_id))
            .run(db)
            .await?;

        if let Some(group) = groups.pop() {
            result.push((gm, group));
        }
    }

    Ok(result)
}
