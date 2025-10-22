use crate::app_data::AppData;
use crate::common::json_error::ToJsonError;
use crate::jwt::token::decode_token;
use crate::models::admin::Admin;
use crate::models::admin_role::AvailableAdminRole;
use crate::models::student::Student;
use actix_web::dev::ServiceRequest;
use actix_web::http::StatusCode;
use actix_web::{web, Error, HttpMessage};
use log::{error, warn};
use std::collections::HashSet;
use welds::state::DbState;

pub(crate) const ADMIN_HEADER_NAME: &str = "X-Admin-Token";
pub(crate) const STUDENT_HEADER_NAME: &str = "X-Student-Token";

// Authority constants
pub(crate) const ROLE_ADMIN_ROOT: &str = "ROLE_ADMIN_ROOT";
pub(crate) const ROLE_ADMIN_PROFESSOR: &str = "ROLE_ADMIN_PROFESSOR";
pub(crate) const ROLE_ADMIN_COORDINATOR: &str = "ROLE_ADMIN_COORDINATOR";
pub(crate) const ROLE_STUDENT: &str = "ROLE_STUDENT";

/// Extracts authorities from the request for actix-web-grants.
/// This function:
/// 1. Extracts JWT token from request headers
/// 2. Decodes and validates the token
/// 3. Loads the user (Admin or Student) from the database
/// 4. Stores the user in request extensions
/// 5. Returns a HashSet of role authorities
pub async fn extract(req: &ServiceRequest) -> Result<HashSet<String>, Error> {
    const INVALID_TOKEN: &str = "Invalid token";

    // Try to extract token from either header
    let token = req
        .headers()
        .get(ADMIN_HEADER_NAME)
        .or_else(|| req.headers().get(STUDENT_HEADER_NAME))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Return empty authorities if no token found (public endpoints)
    let Some(token) = token else {
        return Ok(HashSet::new());
    };

    // Get app data
    let app_state = req
        .app_data::<web::Data<AppData>>()
        .ok_or_else(|| -> Error {
            error!("app data not found in request");
            "jwt token not provided"
                .to_json_error(StatusCode::UNAUTHORIZED)
                .into()
        })?;

    // Decode token
    let decoded_token =
        decode_token(token, app_state.config.jwt_secret().as_bytes()).map_err(|e| -> Error {
            warn!("unable to decode jwt token: {}", e);
            INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED).into()
        })?;

    let mut authorities = HashSet::new();

    if decoded_token.adm {
        // Admin processing
        let role: AvailableAdminRole = decoded_token.rl.try_into().map_err(|_| {
            warn!("invalid admin role in token: {}", decoded_token.rl);
            INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED)
        })?;

        // Add role-specific authority
        let authority = match role {
            AvailableAdminRole::Root => ROLE_ADMIN_ROOT,
            AvailableAdminRole::Professor => ROLE_ADMIN_PROFESSOR,
            AvailableAdminRole::Coordinator => ROLE_ADMIN_COORDINATOR,
        };
        authorities.insert(authority.to_string());

        // Load admin from database
        let admin = Admin::where_col(|a| a.admin_id.equal(decoded_token.sub))
            .run(&app_state.db)
            .await
            .map_err(|e| {
                error!("unable to fetch admin from database: {}", e);
                "unable to fetch admin from database"
                    .to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .pop()
            .ok_or_else(|| {
                warn!("login attempt with non-existing admin");
                INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED)
            })?;

        let admin = DbState::into_inner(admin);

        // Store admin in request extensions
        req.extensions_mut().insert::<Admin>(admin);
    } else {
        // Student processing
        authorities.insert(ROLE_STUDENT.to_string());

        // Load student from database
        let student = Student::where_col(|s| s.student_id.equal(decoded_token.sub))
            .run(&app_state.db)
            .await
            .map_err(|e| {
                error!("unable to fetch student from database: {}", e);
                "unable to fetch student from database"
                    .to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .pop()
            .ok_or_else(|| {
                warn!("login attempt with non-existing student");
                INVALID_TOKEN.to_json_error(StatusCode::UNAUTHORIZED)
            })?;

        let student = DbState::into_inner(student);

        // Store student in request extensions
        req.extensions_mut().insert::<Student>(student);
    }

    Ok(authorities)
}
