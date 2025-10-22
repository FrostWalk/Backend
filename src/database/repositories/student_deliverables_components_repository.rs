use crate::models::student_deliverable::StudentDeliverable;
use crate::models::student_deliverable_component::StudentDeliverableComponent;
use crate::models::student_deliverables_component::StudentDeliverablesComponent;
use welds::connections::postgres::PostgresClient;
use welds::state::DbState;

/// Get all student deliverables components relationships
pub(crate) async fn get_all(
    db: &PostgresClient,
) -> welds::errors::Result<Vec<DbState<StudentDeliverablesComponent>>> {
    StudentDeliverablesComponent::all().run(db).await
}

/// Get a student deliverables component relationship by its ID
pub(crate) async fn get_by_id(
    db: &PostgresClient, id: i32,
) -> welds::errors::Result<Option<DbState<StudentDeliverablesComponent>>> {
    let mut rows = StudentDeliverablesComponent::where_col(|sdc| sdc.id.equal(id))
        .run(db)
        .await?;

    Ok(rows.pop())
}

/// Get all components for a specific student deliverable
pub(crate) async fn get_by_deliverable_id(
    db: &PostgresClient, deliverable_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverablesComponent>>> {
    StudentDeliverablesComponent::where_col(|sdc| sdc.student_deliverable_id.equal(deliverable_id))
        .run(db)
        .await
}

/// Get all deliverables for a specific component
pub(crate) async fn get_by_component_id(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverablesComponent>>> {
    StudentDeliverablesComponent::where_col(|sdc| {
        sdc.student_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await
}

/// Check if a relationship exists between a deliverable and component
pub(crate) async fn relationship_exists(
    db: &PostgresClient, deliverable_id: i32, component_id: i32,
) -> welds::errors::Result<bool> {
    let rows = StudentDeliverablesComponent::where_col(|sdc| {
        sdc.student_deliverable_id.equal(deliverable_id)
    })
    .where_col(|sdc| sdc.student_deliverable_component_id.equal(component_id))
    .limit(1)
    .run(db)
    .await?;

    Ok(!rows.is_empty())
}

/// Get deliverables with their details for a specific student component
pub(crate) async fn get_deliverables_with_details_for_component(
    db: &PostgresClient, component_id: i32,
) -> welds::errors::Result<
    Vec<(
        DbState<StudentDeliverablesComponent>,
        DbState<StudentDeliverable>,
    )>,
> {
    let relationships = StudentDeliverablesComponent::where_col(|sdc| {
        sdc.student_deliverable_component_id.equal(component_id)
    })
    .run(db)
    .await?;

    let mut result = Vec::new();
    for relationship in relationships {
        let mut deliverables = StudentDeliverable::where_col(|sd| {
            sd.student_deliverable_id
                .equal(relationship.student_deliverable_id)
        })
        .run(db)
        .await?;

        if let Some(deliverable) = deliverables.pop() {
            result.push((relationship, deliverable));
        }
    }

    Ok(result)
}

/// Get components with their details for a specific student deliverable
pub(crate) async fn get_components_with_details_for_deliverable(
    db: &PostgresClient, deliverable_id: i32,
) -> welds::errors::Result<
    Vec<(
        DbState<StudentDeliverablesComponent>,
        DbState<StudentDeliverableComponent>,
    )>,
> {
    let relationships = StudentDeliverablesComponent::where_col(|sdc| {
        sdc.student_deliverable_id.equal(deliverable_id)
    })
    .run(db)
    .await?;

    let mut result = Vec::new();
    for relationship in relationships {
        let mut components = StudentDeliverableComponent::where_col(|sc| {
            sc.student_deliverable_component_id
                .equal(relationship.student_deliverable_component_id)
        })
        .run(db)
        .await?;

        if let Some(component) = components.pop() {
            result.push((relationship, component));
        }
    }
    Ok(result)
}

/// Delete a student deliverables component relationship by ID
pub(crate) async fn delete_by_id(
    db: &PostgresClient, relationship_id: i32,
) -> welds::errors::Result<()> {
    StudentDeliverablesComponent::where_col(|sdc| sdc.id.equal(relationship_id))
        .delete(db)
        .await?;
    Ok(())
}

/// Create a new student deliverables component relationship
pub(crate) async fn create(
    db: &PostgresClient, student_deliverables_component: StudentDeliverablesComponent,
) -> welds::errors::Result<DbState<StudentDeliverablesComponent>> {
    let mut state = DbState::new_uncreated(student_deliverables_component);
    state.save(db).await?;
    Ok(state)
}

/// Update a student deliverables component relationship
pub(crate) async fn update(
    db: &PostgresClient, mut state: DbState<StudentDeliverablesComponent>,
) -> welds::errors::Result<DbState<StudentDeliverablesComponent>> {
    state.save(db).await?;
    Ok(state)
}

/// Get components for a specific student deliverable
pub(crate) async fn get_components_for_deliverable(
    db: &PostgresClient, deliverable_id: i32,
) -> welds::errors::Result<Vec<DbState<StudentDeliverablesComponent>>> {
    StudentDeliverablesComponent::where_col(|sdc| sdc.student_deliverable_id.equal(deliverable_id))
        .run(db)
        .await
}
