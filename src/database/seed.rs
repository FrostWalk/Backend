use crate::models::admin_role::{AdminRole, AvailableAdminRole};
use crate::models::student_role::{AvailableStudentRole, StudentRole};
use welds::state::DbState;

/// Seeds the admin roles table with the default roles
pub(crate) async fn seed_admin_roles(db: &impl welds::Client) -> welds::errors::Result<()> {
    let roles: &[(i32, &str)] = &[
        (AvailableAdminRole::Root as i32, "Root"),
        (AvailableAdminRole::Professor as i32, "Professor"),
        (AvailableAdminRole::Coordinator as i32, "Coordinator"),
    ];

    for (id, name) in roles {
        let mut rows = AdminRole::where_col(|r| r.admin_role_id.equal(*id))
            .limit(1)
            .run(db)
            .await?;

        if let Some(mut state) = rows.pop() {
            if state.name != *name {
                state.name = (*name).to_string();
                state.save(db).await?;
            }
        } else {
            // not exists: insert
            let mut state = DbState::new_uncreated(AdminRole {
                admin_role_id: *id,
                name: (*name).to_string(),
            });
            state.save(db).await?;
        }
    }

    Ok(())
}

/// Seeds the student roles table with the default roles
pub(crate) async fn seed_student_roles(db: &impl welds::Client) -> welds::errors::Result<()> {
    let roles: &[(i32, &str)] = &[
        (AvailableStudentRole::GroupLeader as i32, "Group Leader"),
        (AvailableStudentRole::Member as i32, "Member"),
    ];

    for (id, name) in roles {
        let mut rows = StudentRole::where_col(|r| r.student_role_id.equal(*id))
            .limit(1)
            .run(db)
            .await?;

        if let Some(mut state) = rows.pop() {
            if state.name != *name {
                state.name = (*name).to_string();
                state.save(db).await?;
            }
        } else {
            // not exists: insert
            let mut state = DbState::new_uncreated(StudentRole {
                student_role_id: *id,
                name: (*name).to_string(),
            });
            state.save(db).await?;
        }
    }

    Ok(())
}

/// Seeds all roles (admin and student) in the database
pub(crate) async fn seed_all_roles(db: &impl welds::Client) -> welds::errors::Result<()> {
    seed_admin_roles(db).await?;
    seed_student_roles(db).await?;
    Ok(())
}
