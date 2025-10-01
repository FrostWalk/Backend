use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id_and_payload, JsonError, ToJsonError};
use crate::database::repositories::security_codes::security_code_exists;
use crate::models::security_code::SecurityCode;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{DateTime, Duration, Utc};
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
pub(crate) struct CreateCodeScheme {
    #[schema(example = 10)]
    pub project_id: i32,
    #[schema(value_type = String, example = "2025-09-22T12:34:56Z")]
    pub expiration: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateCodeResponse {
    #[schema(example = "D3K-Z9A")]
    code: String,
}

#[utoipa::path(
    post,
    path = "/v1/admins/security-codes",
    request_body = CreateCodeScheme,
    responses(
        (status = 201, description = "Code created successfully", body = CreateCodeResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 500, description = "Internal server error", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Security codes management",
)]
/// Generate a unique code for a project
pub(in crate::api::v1) async fn create_code_handler(
    req: Json<CreateCodeScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let skew = Duration::days(1);
    let now = Utc::now() - skew;

    if req.project_id <= 0 {
        return Err("Project id field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    } else if req.expiration <= now {
        return Err("Expiration must be grater than one day".to_json_error(StatusCode::BAD_REQUEST));
    }

    let mut done = false;
    let mut code = String::new();
    while !done {
        code = generate_random_code();
        match security_code_exists(&data.db, code.as_str()).await {
            Ok(exists) => done = !exists,
            Err(e) => {
                return Err(error_with_log_id_and_payload(
                    format!(
                        "unable to check if security code {:?} exists in database. Error: {}",
                        code, e
                    ),
                    "Failed to create security code",
                    StatusCode::INTERNAL_SERVER_ERROR,
                    log::Level::Error,
                    &req,
                ));
            }
        }
    }

    // Create and save the security code to the database
    let mut security_code_state = DbState::new_uncreated(SecurityCode {
        security_code_id: 0,
        project_id: req.project_id,
        code: code.clone(),
        expiration: req.expiration,
    });

    match security_code_state.save(&data.db).await {
        Ok(_) => Ok(HttpResponse::Created().json(CreateCodeResponse { code })),
        Err(e) => Err(error_with_log_id_and_payload(
            format!("unable to save security code to database: {}", e),
            "Failed to create security code",
            StatusCode::INTERNAL_SERVER_ERROR,
            log::Level::Error,
            &req,
        )),
    }
}
