use crate::app_data::AppData;
use actix_web::web::Data;
use actix_web::{HttpResponse, Result};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct AllowedDomainsResponse {
    /// List of email domains allowed for account creation
    #[schema(example = json!(["unitn.it", "studenti.unitn.it"]))]
    domains: Vec<String>,
}

/// Get allowed email domains for student registration
///
/// This endpoint returns the list of email domains that are allowed
/// for creating student accounts in the system. Use this endpoint
/// before signup to validate if a user's email domain is acceptable.
#[utoipa::path(
    get,
    path = "/v1/students/auth/allowed-domains",
    tag = "Student authentication",
    responses(
        (status = 200, description = "List of allowed email domains", body = AllowedDomainsResponse)
    ),
    summary = "Get allowed email domains for registration",
    description = "Returns a list of email domains that can be used to create student accounts. This endpoint does not require authentication."
)]
pub(super) async fn allowed_domains_handler(data: Data<AppData>) -> Result<HttpResponse> {
    let domains = data.config.allowed_signup_domains().clone();

    let response = AllowedDomainsResponse { domains };

    Ok(HttpResponse::Ok().json(response))
}
