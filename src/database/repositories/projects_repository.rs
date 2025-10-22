use crate::models::group_deliverable::GroupDeliverable;
use crate::models::group_deliverable_component::GroupDeliverableComponent;
use crate::models::project::Project;
use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all projects from the database
pub(crate) async fn get_all(db: &PostgresClient) -> welds::errors::Result<Vec<DbState<Project>>> {
    Project::all().run(db).await
}

/// Get a project by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<Option<DbState<Project>>> {
    let mut rows = Project::where_col(|p| p.project_id.equal(project_id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Delete a project by its ID
/// Returns true if the project was deleted, false if not found
pub(crate) async fn delete_by_id(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<bool> {
    let mut rows = Project::where_col(|p| p.project_id.equal(project_id))
        .run(db)
        .await?;

    if let Some(mut state) = rows.pop() {
        state.delete(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Create a new project
pub(crate) async fn create(
    db: &PostgresClient, project: Project,
) -> welds::errors::Result<DbState<Project>> {
    let mut state = DbState::new_uncreated(project);
    state.save(db).await?;
    Ok(state)
}

/// Update a project by ID
pub(crate) async fn update_by_id(
    db: &PostgresClient, project_id: i32, name: Option<String>, max_student_uploads: Option<i32>,
    max_group_size: Option<i32>, active: Option<bool>,
) -> welds::errors::Result<()> {
    if let Some(name) = name {
        Project::where_col(|p| p.project_id.equal(project_id))
            .set(|p| p.name, name)
            .run(db)
            .await?;
    }
    if let Some(uploads) = max_student_uploads {
        Project::where_col(|p| p.project_id.equal(project_id))
            .set(|p| p.max_student_uploads, uploads)
            .run(db)
            .await?;
    }
    if let Some(size) = max_group_size {
        Project::where_col(|p| p.project_id.equal(project_id))
            .set(|p| p.max_group_size, size)
            .run(db)
            .await?;
    }
    if let Some(active) = active {
        Project::where_col(|p| p.project_id.equal(project_id))
            .set(|p| p.active, active)
            .run(db)
            .await?;
    }
    Ok(())
}

/// Update a project
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<Project>,
) -> welds::errors::Result<DbState<Project>> {
    state.save(db).await?;
    Ok(state)
}

/// Get project details with all related entities
pub(crate) async fn get_project_details(
    db: &PostgresClient, project_id: i32,
) -> welds::errors::Result<
    Option<(
        DbState<Project>,
        Vec<DbState<GroupDeliverable>>,
        Vec<DbState<GroupDeliverableComponent>>,
        Vec<DbState<StudentDeliverable>>,
        Vec<DbState<StudentDeliverableComponent>>,
    )>,
> {
    // Get the project
    let project_state = match get_by_id(db, project_id).await? {
        Some(state) => state,
        None => return Ok(None),
    };

    // Get group deliverables
    let group_deliverables = Project::where_col(|p| p.project_id.equal(project_id))
        .map_query(|p| p.group_deliverables)
        .run(db)
        .await?;

    // Get group components
    let group_components = Project::where_col(|p| p.project_id.equal(project_id))
        .map_query(|p| p.group_deliverable_components)
        .run(db)
        .await?;

    // Get student deliverables
    let student_deliverables = Project::where_col(|p| p.project_id.equal(project_id))
        .map_query(|p| p.student_deliverables)
        .run(db)
        .await?;

    // Get student components
    let student_components = Project::where_col(|p| p.project_id.equal(project_id))
        .map_query(|p| p.student_deliverable_components)
        .run(db)
        .await?;

    Ok(Some((
        project_state,
        group_deliverables,
        group_components,
        student_deliverables,
        student_components,
    )))
}

/// Get projects with all related entities for a student
pub(crate) async fn get_projects_with_details_for_student(
    db: &PostgresClient, student_id: i32,
) -> welds::errors::Result<
    Vec<(
        DbState<Project>,
        Vec<DbState<GroupDeliverable>>,
        Vec<DbState<GroupDeliverableComponent>>,
        Vec<DbState<StudentDeliverable>>,
        Vec<DbState<StudentDeliverableComponent>>,
    )>,
> {
    use crate::models::group_member::GroupMember;

    // Get projects through group membership
    let projects = GroupMember::where_col(|gm| gm.student_id.equal(student_id))
        .map_query(|gm| gm.group)
        .map_query(|g| g.project)
        .run(db)
        .await?;

    let mut result = Vec::new();

    for project in projects {
        let project_id = project.project_id;

        // Get group deliverables
        let group_deliverables = Project::where_col(|p| p.project_id.equal(project_id))
            .map_query(|p| p.group_deliverables)
            .run(db)
            .await?;

        // Get group components
        let group_components = Project::where_col(|p| p.project_id.equal(project_id))
            .map_query(|p| p.group_deliverable_components)
            .run(db)
            .await?;

        // Get student deliverables
        let student_deliverables = Project::where_col(|p| p.project_id.equal(project_id))
            .map_query(|p| p.student_deliverables)
            .run(db)
            .await?;

        // Get student components
        let student_components = Project::where_col(|p| p.project_id.equal(project_id))
            .map_query(|p| p.student_deliverable_components)
            .run(db)
            .await?;

        result.push((
            project,
            group_deliverables,
            group_components,
            student_deliverables,
            student_components,
        ));
    }

    Ok(result)
}
