use crate::app_data::AppData;
use crate::common::json_error::{
    error_with_log_id, error_with_log_id_and_payload, JsonError, ToJsonError,
};
use crate::database::repositories::coordinator_projects_repository;
use crate::database::repositories::security_codes::{
    get_by_id, security_code_exists, update as update_security_code,
};
use crate::jwt::get_user::LoggedUser;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::{schema, ToSchema};
use welds::state::DbState;

fn generate_random_code() -> String {
    use rand::Rng;

    let mut rng = rand::rng();
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut s = String::with_capacity(7);

    for i in 0..6 {
        if i == 3 {
            s.push('-');
        }
        let idx = rng.random_range(0..CHARS.len());
        s.push(CHARS[idx] as char);
    }

    s
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(crate) struct UpdateCodeScheme {
    #[schema(example = "D3K-Z9A")]
    pub code: Option<String>,
    #[schema(value_type = String, example = "2025-09-22T12:34:56Z")]
    pub expiration: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UpdateCodeResponse {
    #[schema(example = 1)]
    pub security_code_id: i32,
    #[schema(example = "D3K-Z9A")]
    pub code: String,
    #[schema(value_type = String, example = "2025-09-22T12:34:56Z")]
    pub expiration: DateTime<Utc>,
    #[schema(example = 10)]
    pub project_id: i32,
}

#[utoipa::path(
    patch,
    path = "/v1/admins/security-codes/{security_code_id}",
    request_body = UpdateCodeScheme,
    responses(
        (status = 200, description = "Code updated successfully", body = UpdateCodeResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 403, description = "Access denied", body = JsonError),
        (status = 404, description = "Security code not found", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Security codes management",
)]
/// Partially update a security code
///
/// Coordinators can only update codes for projects they are assigned to. Professors/Root can update codes for any project.
/// If code is provided, a new unique code will be generated. If expiration is provided, it must be greater than one day from now.
pub(in crate::api::v1) async fn update_code_handler(
    req: HttpRequest, 
    path: Path<i32>, 
    body: Json<UpdateCodeScheme>, 
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let security_code_id = path.into_inner();

    // Get the existing security code
    let existing_code = match get_by_id(&data.db, security_code_id).await {
        Ok(Some(code)) => code,
        Ok(None) => {
            return Err("Security code not found".to_json_error(StatusCode::NOT_FOUND));
        }
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to retrieve security code from database: {}", e),
                "Failed to update security code",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let existing_code_data = DbState::into_inner(existing_code);

    // Check if user is a coordinator and if they have access to this project
    let is_coordinator = user.admin_role_id == AvailableAdminRole::Coordinator as i32;
    if is_coordinator {
        let is_assigned = coordinator_projects_repository::is_assigned(
            &data.db,
            user.admin_id,
            existing_code_data.project_id,
        )
        .await
        .map_err(|e| {
            error_with_log_id(
                format!("unable to check coordinator assignment: {}", e),
                "Failed to update security code",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            )
        })?;

        if !is_assigned {
            return Err("Access denied - you are not assigned to this project"
                .to_json_error(StatusCode::FORBIDDEN));
        }
    }

    // Validate expiration if provided
    if let Some(expiration) = body.expiration {
        let skew = Duration::days(1);
        let now = Utc::now() - skew;

        if expiration <= now {
            return Err(
                "Expiration must be greater than one day".to_json_error(StatusCode::BAD_REQUEST)
            );
        }
    }

    // Generate new code if requested
    let new_code = if body.code.is_some() {
        let mut done = false;
        let mut code = String::new();
        while !done {
            code = generate_random_code();
            match security_code_exists(&data.db, code.as_str()).await {
                Ok(exists) => {
                    // Allow the same code if it's the current code
                    if !exists || code == existing_code_data.code {
                        done = true;
                    }
                }
                Err(e) => {
                    return Err(error_with_log_id_and_payload(
                        format!(
                            "unable to check if security code {:?} exists in database. Error: {}",
                            code, e
                        ),
                        "Failed to update security code",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        log::Level::Error,
                        &body,
                    ));
                }
            }
        }
        code
    } else {
        existing_code_data.code.clone()
    };

    // Update the security code
    let final_expiration = body.expiration.unwrap_or(existing_code_data.expiration);

    match update_security_code(
        &data.db,
        security_code_id,
        new_code.clone(),
        final_expiration,
    )
    .await
    {
        Ok(Some(_)) => Ok(HttpResponse::Ok().json(UpdateCodeResponse {
            security_code_id,
            code: new_code,
            expiration: final_expiration,
            project_id: existing_code_data.project_id,
        })),
        Ok(None) => Err("Security code not found".to_json_error(StatusCode::NOT_FOUND)),
        Err(e) => Err(error_with_log_id_and_payload(
            format!("unable to update security code in database: {}", e),
            "Failed to update security code",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &body,
        )),
    }
}
