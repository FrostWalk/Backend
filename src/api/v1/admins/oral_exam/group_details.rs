use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::database::repositories::{
    complaints_repository, group_component_implementation_details_repository,
    group_deliverable_components_repository, group_deliverable_selections_repository,
    group_deliverables_repository, groups_repository, oral_exam_repository, projects_repository,
    student_deliverable_components_repository, student_deliverable_selections_repository,
    student_deliverables_components_repository, student_deliverables_repository,
    student_uploads_repository, students_repository,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::transaction::Transaction;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use welds::state::DbState;

// ── Response types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OralExamGroupDetailsResponse {
    pub group_id: i32,
    pub name: String,
    pub project_id: i32,
    pub project_name: String,
    pub members: Vec<OralExamMemberDetail>,
    pub group_deliverable: Option<OralExamGroupDeliverable>,
    pub complaints_filed: Vec<ComplaintSummary>,
    pub complaints_received: Vec<ComplaintSummary>,
    pub fair_sales: Vec<FairSaleSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OralExamMemberDetail {
    pub student_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub university_id: i32,
    pub is_leader: bool,
    pub student_deliverable: Option<StudentDeliverableSummary>,
    pub upload_count: Option<i32>,
    pub oral_exam_note: Option<String>,
    pub oral_exam_note_updated_at: Option<DateTime<Utc>>,
    pub oral_exam_completed: bool,
    pub oral_exam_completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentDeliverableSummary {
    pub student_deliverable_id: i32,
    pub name: String,
    pub components: Vec<StudentComponentSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct StudentComponentSummary {
    pub student_deliverable_component_id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OralExamGroupDeliverable {
    pub group_deliverable_id: i32,
    pub name: String,
    pub implementation_details: Vec<ImplementationDetailSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ImplementationDetailSummary {
    pub id: i32,
    pub component_name: String,
    pub markdown_description: String,
    pub repository_link: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ComplaintSummary {
    pub complaint_id: i32,
    pub transaction_id: i32,
    pub other_group_id: i32,
    pub other_group_name: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct FairSaleSummary {
    pub transaction_id: i32,
    pub buyer_group_id: i32,
    pub buyer_group_name: String,
    pub component_name: String,
    pub timestamp: DateTime<Utc>,
}

// ── Handler ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/v1/admins/oral-exam/projects/{project_id}/groups/{group_id}",
    responses(
        (status = 200, description = "Full oral exam group details", body = OralExamGroupDetailsResponse),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 404, description = "Group or project not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin Oral Exam",
)]
#[actix_web_grants::protect(any("ROLE_ADMIN_ROOT", "ROLE_ADMIN_PROFESSOR"))]
pub(super) async fn get_oral_exam_group_details(
    req: HttpRequest, path: Path<(i32, i32)>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let _admin = match req.extensions().get_admin() {
        Ok(admin) => admin,
        Err(_) => {
            return Err(error_with_log_id(
                "entered a protected route without an admin loaded in the request",
                "Authentication error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let (project_id, group_id) = path.into_inner();

    let project = DbState::into_inner(
        projects_repository::get_by_id(&data.db, project_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch project {}: {}", project_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!("project {} not found", project_id),
                    "Project not found",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })?,
    );

    let group = DbState::into_inner(
        groups_repository::get_by_id(&data.db, group_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch group {}: {}", group_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?
            .ok_or_else(|| {
                error_with_log_id(
                    format!("group {} not found", group_id),
                    "Group not found",
                    StatusCode::NOT_FOUND,
                    log::Level::Warn,
                )
            })?,
    );

    if group.project_id != project_id {
        return Err(error_with_log_id(
            format!(
                "group {} does not belong to project {}",
                group_id, project_id
            ),
            "Group not found",
            StatusCode::NOT_FOUND,
            log::Level::Warn,
        ));
    }

    // ── Members ──────────────────────────────────────────────────────────

    let raw_members = groups_repository::get_group_members(&data.db, group_id)
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to fetch members for group {}: {}", group_id, e),
                "Database error",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

    let mut members: Vec<OralExamMemberDetail> = Vec::new();

    for member_state in raw_members {
        let member = DbState::into_inner(member_state);

        let student_state = students_repository::get_by_id(&data.db, member.student_id)
            .await
            .map_err(|e| {
                error_with_log_id(
                    format!("unable to fetch student {}: {}", member.student_id, e),
                    "Database error",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                )
            })?;

        let Some(student_state) = student_state else {
            continue;
        };
        let student = DbState::into_inner(student_state);

        let is_leader = member.student_role_id == 1;

        // Student deliverable selection for this project
        let student_deliverable =
            match student_deliverable_selections_repository::get_by_student_and_project(
                &data.db,
                student.student_id,
                project_id,
            )
            .await
            {
                Ok(Some(sel_state)) => {
                    let sel = DbState::into_inner(sel_state);
                    let deliverable_state = student_deliverables_repository::get_by_id(
                        &data.db,
                        sel.student_deliverable_id,
                    )
                    .await
                    .ok()
                    .flatten();

                    if let Some(d_state) = deliverable_state {
                        let d = DbState::into_inner(d_state);

                        let relations = student_deliverables_components_repository::get_components_for_deliverable(
                        &data.db,
                        sel.student_deliverable_id,
                    )
                    .await
                    .unwrap_or_default();

                        let mut components = Vec::new();
                        for rel_state in relations {
                            let rel = DbState::into_inner(rel_state);
                            if let Ok(Some(comp_state)) =
                                student_deliverable_components_repository::get_by_id(
                                    &data.db,
                                    rel.student_deliverable_component_id,
                                )
                                .await
                            {
                                let comp = DbState::into_inner(comp_state);
                                components.push(StudentComponentSummary {
                                    student_deliverable_component_id: comp
                                        .student_deliverable_component_id,
                                    name: comp.name,
                                });
                            }
                        }

                        Some(StudentDeliverableSummary {
                            student_deliverable_id: d.student_deliverable_id,
                            name: d.name,
                            components,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

        // Upload count
        let upload_count =
            match student_deliverable_selections_repository::get_by_student_and_project(
                &data.db,
                student.student_id,
                project_id,
            )
            .await
            {
                Ok(Some(sel_state)) => {
                    let sel = DbState::into_inner(sel_state);
                    match student_uploads_repository::get_by_selection_id(
                        &data.db,
                        sel.student_deliverable_selection_id,
                    )
                    .await
                    {
                        Ok(Some(upload_state)) => Some(upload_state.upload_count),
                        _ => None,
                    }
                }
                _ => None,
            };

        // Oral exam note
        let note_state = oral_exam_repository::get_note(&data.db, student.student_id, project_id)
            .await
            .ok()
            .flatten();
        let (oral_exam_note, oral_exam_note_updated_at) = match note_state {
            Some(n) => (Some(n.note_text.clone()), Some(n.updated_at)),
            None => (None, None),
        };

        // Oral exam completion
        let completion_state =
            oral_exam_repository::get_completion(&data.db, student.student_id, project_id)
                .await
                .ok()
                .flatten();
        let (oral_exam_completed, oral_exam_completed_at) = match completion_state {
            Some(c) => (true, Some(c.completed_at)),
            None => (false, None),
        };

        members.push(OralExamMemberDetail {
            student_id: student.student_id,
            first_name: student.first_name,
            last_name: student.last_name,
            email: student.email,
            university_id: student.university_id,
            is_leader,
            student_deliverable,
            upload_count,
            oral_exam_note,
            oral_exam_note_updated_at,
            oral_exam_completed,
            oral_exam_completed_at,
        });
    }

    // ── Group deliverable ─────────────────────────────────────────────────

    let group_deliverable =
        match group_deliverable_selections_repository::get_by_group_id(&data.db, group_id).await {
            Ok(Some(sel_state)) => {
                let sel = DbState::into_inner(sel_state);
                match group_deliverables_repository::get_by_id(&data.db, sel.group_deliverable_id)
                    .await
                {
                    Ok(Some(d_state)) => {
                        let d = DbState::into_inner(d_state);

                        let details =
                            group_component_implementation_details_repository::get_by_selection_id(
                                &data.db,
                                sel.group_deliverable_selection_id,
                            )
                            .await
                            .unwrap_or_default();

                        let mut implementation_details = Vec::new();
                        for detail_state in details {
                            let detail = DbState::into_inner(detail_state);
                            let component_name =
                                match group_deliverable_components_repository::get_by_id(
                                    &data.db,
                                    detail.group_deliverable_component_id,
                                )
                                .await
                                {
                                    Ok(Some(c_state)) => DbState::into_inner(c_state).name,
                                    _ => format!(
                                        "Component {}",
                                        detail.group_deliverable_component_id
                                    ),
                                };
                            implementation_details.push(ImplementationDetailSummary {
                                id: detail.id,
                                component_name,
                                markdown_description: detail.markdown_description,
                                repository_link: detail.repository_link,
                            });
                        }

                        Some(OralExamGroupDeliverable {
                            group_deliverable_id: d.group_deliverable_id,
                            name: d.name,
                            implementation_details,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        };

    // ── Complaints ────────────────────────────────────────────────────────

    let raw_filed = complaints_repository::get_filed_by_group(&data.db, group_id)
        .await
        .unwrap_or_default();
    let raw_received = complaints_repository::get_received_by_group(&data.db, group_id)
        .await
        .unwrap_or_default();

    let mut complaints_filed = Vec::new();
    for c_state in raw_filed {
        let c = DbState::into_inner(c_state);
        let other_name = group_name(&data.db, c.to_group_id).await;
        complaints_filed.push(ComplaintSummary {
            complaint_id: c.complaint_id,
            transaction_id: c.transaction_id,
            other_group_id: c.to_group_id,
            other_group_name: other_name,
            text: c.text,
            created_at: c.created_at,
        });
    }

    let mut complaints_received = Vec::new();
    for c_state in raw_received {
        let c = DbState::into_inner(c_state);
        let other_name = group_name(&data.db, c.from_group_id).await;
        complaints_received.push(ComplaintSummary {
            complaint_id: c.complaint_id,
            transaction_id: c.transaction_id,
            other_group_id: c.from_group_id,
            other_group_name: other_name,
            text: c.text,
            created_at: c.created_at,
        });
    }

    // ── Fair sales (this group as seller) ─────────────────────────────────

    let fair_sales = match &group_deliverable {
        Some(_gd) => {
            // Find the group_deliverable_selection_id to look up sales
            match group_deliverable_selections_repository::get_by_group_id(&data.db, group_id).await
            {
                Ok(Some(sel_state)) => {
                    let sel = DbState::into_inner(sel_state);
                    let sales =
                        fetch_sales_for_selection(&data.db, sel.group_deliverable_selection_id)
                            .await;
                    sales
                }
                _ => Vec::new(),
            }
        }
        None => Vec::new(),
    };

    Ok(HttpResponse::Ok().json(OralExamGroupDetailsResponse {
        group_id,
        name: group.name,
        project_id,
        project_name: project.name,
        members,
        group_deliverable,
        complaints_filed,
        complaints_received,
        fair_sales,
    }))
}

async fn group_name(db: &welds::connections::postgres::PostgresClient, group_id: i32) -> String {
    match groups_repository::get_by_id(db, group_id).await {
        Ok(Some(g)) => g.name.clone(),
        _ => format!("Group {}", group_id),
    }
}

async fn fetch_sales_for_selection(
    db: &welds::connections::postgres::PostgresClient, selection_id: i32,
) -> Vec<FairSaleSummary> {
    let rows: Result<Vec<DbState<Transaction>>, _> =
        Transaction::where_col(|t| t.group_deliverable_selection_id.equal(selection_id))
            .run(db)
            .await;

    let transactions = match rows {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut sales = Vec::new();
    for tx_state in transactions {
        let tx = DbState::into_inner(tx_state);

        let buyer_name = group_name(db, tx.buyer_group_id).await;

        let component_name = match group_deliverable_components_repository::get_by_id(
            db,
            tx.group_deliverable_component_id,
        )
        .await
        {
            Ok(Some(c)) => c.name.clone(),
            _ => format!("Component {}", tx.group_deliverable_component_id),
        };

        sales.push(FairSaleSummary {
            transaction_id: tx.transaction_id,
            buyer_group_id: tx.buyer_group_id,
            buyer_group_name: buyer_name,
            component_name,
            timestamp: tx.timestamp,
        });
    }
    sales
}
