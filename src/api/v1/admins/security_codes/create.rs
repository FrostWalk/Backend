use crate::app_data::AppData;
use crate::common::json_error::{database_error, JsonError, ToJsonError};
use crate::database::repositories::security_codes::security_code_exists;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use chrono::{DateTime, Duration, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::{schema, ToSchema};

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

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateCodeScheme {
    #[schema(example = 10)]
    pub project_id: i32,
    #[schema(example = 10)]
    pub user_role_id: i32,
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
    payload: Json<CreateCodeScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let payload = payload.into_inner();

    let skew = Duration::days(1);
    let now = Utc::now() - skew;

    if payload.project_id <= 0 {
        return Err("Project id field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    } else if payload.user_role_id <= 0 {
        return Err("User role id field is mandatory".to_json_error(StatusCode::BAD_REQUEST));
    } else if payload.expiration <= now {
        return Err("Expiration must be grater than one day".to_json_error(StatusCode::BAD_REQUEST));
    }

    let mut done = false;
    let mut code = String::new();
    while !done {
        code = generate_random_code();
        match security_code_exists(&data.db, code.as_str()).await {
            Ok(b) => done = b,
            Err(e) => {
                error!(
                    "unable to check if security code {:?} exists in database. Error: {e}",
                    code
                );
                return Err(database_error());
            }
        }
    }

    Ok(HttpResponse::Created().json(CreateCodeResponse { code }))
}
