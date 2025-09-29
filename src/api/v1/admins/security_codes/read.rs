use crate::app_data::AppData;
use crate::common::json_error::{error_with_log_id, JsonError};
use crate::models::security_code::SecurityCode;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use welds::prelude::DbState;
#[derive(Debug, Serialize, ToSchema)]
pub struct SecurityCodeWithNames {
    pub security_code_id: i32,
    pub code: String,
    pub expiration: DateTime<Utc>,
    pub project_id: i32,
    pub project_name: String,
}
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllCodesResponse {
    codes: Vec<SecurityCodeWithNames>,
}
#[utoipa::path(
    get,
    path = "/v1/admins/security-codes",
    responses(
        (status = 200, description = "Found codes", body = GetAllCodesResponse),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Security codes management",
)]
/// Get all security codes
pub(in crate::api::v1) async fn get_all_codes_handler(
    data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let now: DateTime<Utc> = Utc::now();

    let codes: Vec<DbState<SecurityCode>> = match SecurityCode::all()
        .order_by_asc(|sc| sc.security_code_id)
        .run(&data.db)
        .await
    {
        // Weelds currently does not support filtering rows using datetime operators like less_than
        Ok(c) => c.into_iter().filter(|code| code.expiration > now).collect(),
        Err(e) => {
            return Err(error_with_log_id(
                format!(
                    "unable to retrieve security codes from database. Error: {}",
                    e
                ),
                "Failed to retrieve security codes",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let projects = match SecurityCode::all()
        .order_by_asc(|sc| sc.security_code_id)
        .map_query(|sc| sc.project)
        .run(&data.db)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return Err(error_with_log_id(
                format!("unable to retrieve projects from database. Error: {}", e),
                "Failed to retrieve security codes",
                StatusCode::INTERNAL_SERVER_ERROR,
                log::Level::Error,
            ));
        }
    };

    let mut out = Vec::with_capacity(codes.len());
    for (sc_state, p_state) in codes.into_iter().zip(projects) {
        let sc = DbState::into_inner(sc_state);
        let p = DbState::into_inner(p_state);

        out.push(SecurityCodeWithNames {
            security_code_id: sc.security_code_id,
            code: sc.code,
            expiration: sc.expiration,
            project_id: sc.project_id,
            project_name: p.name,
        });
    }

    Ok(HttpResponse::Ok().json(GetAllCodesResponse { codes: out }))
}
