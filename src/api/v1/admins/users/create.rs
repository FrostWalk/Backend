use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repositories::admins_repository::AdminRole;
use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::get_user::LoggedUser;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use entity::admins;
use log::{error, warn};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateAdminScheme {
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "john.doe@example.com")]
    pub email: String,
    #[schema(example = "SecureP@ss123")]
    pub password: String,
    #[schema(example = "2")]
    pub admin_role_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct CreateAdminResponse {
    #[schema(example = "12345")]
    pub admin_id: i32,
}
#[utoipa::path(
    post,
    path = "/v1/admins/users",
    request_body = CreateAdminScheme,
    responses(
        (status = 200, description = "Admin created successfully", body = CreateAdminResponse),
        (status = 400, description = "Invalid data in request", body = JsonError),
        (status = 401, description = "Authentication required", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    security(("AdminAuth" = [])),
    tag = "Admin users management",
)]
/// Creates a new admin user.
///
/// This endpoint allows authenticated users to create new admin accounts. Only users with the root role can create other root users.
pub(super) async fn create_admin_handler(
    req: HttpRequest, payload: Json<CreateAdminScheme>, data: Data<AppData>,
) -> Result<HttpResponse, JsonError> {
    let scheme = payload.into_inner();

    let user = match req.extensions().get_admin() {
        Ok(user) => user,
        Err(e) => {
            error!("entered a protected route without a user loaded in the request");
            return Err(e.to_json_error(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    // only root can create root users
    if (user.admin_role_id != AdminRole::Root as i32)
        && (scheme.admin_role_id == AdminRole::Root as i32)
    {
        warn!("The user {} tried to create a root user", user.email);
        return Err("operation not permitted".to_json_error(StatusCode::FORBIDDEN));
    }

    let adm = admins::ActiveModel {
        admin_id: ActiveValue::NotSet,
        first_name: ActiveValue::Set(scheme.first_name),
        last_name: ActiveValue::Set(scheme.last_name),
        email: ActiveValue::Set(scheme.email),
        password_hash: ActiveValue::Set(password_auth::generate_hash(scheme.password)),
        admin_role_id: ActiveValue::Set(scheme.admin_role_id),
    };

    let result = match data.repositories.admins.create(adm).await {
        Ok(r) => r,
        Err(e) => {
            error!("unable to create admin: {}", e);
            return Err(
                "unable to create admin scheme".to_json_error(StatusCode::INTERNAL_SERVER_ERROR)
            );
        }
    };

    Ok(HttpResponse::Ok().json(CreateAdminResponse {
        admin_id: result.last_insert_id,
    }))
}
