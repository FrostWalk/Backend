use crate::api::v1::admins::users::AdminResponseScheme;
use crate::app_data::AppData;
use crate::common::json_error::{JsonError, ToJsonError};
use crate::database::repositories::admins_repository::AdminRole;
use crate::database::repository_methods_trait::RepositoryMethods;
use crate::jwt::get_user::LoggedUser;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::admins;
use log::error;
use sea_orm::ColumnTrait;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct GetAllAdminsResponse {
    #[schema(example = json!(
         [
            {
                "id": 1,
                "name": "John Doe",
                "email": "john.doe@example.com",
                "admin_role_id": 1
            },
            {
                "id": 2,
                "name": "Jane Smith",
                "email": "jane.smith@example.com",
                "admin_role_id": 2
            }
        ]
    ))]
    pub admins: Vec<AdminResponseScheme>,
}
#[utoipa::path(
    get,
    path = "/v1/admins/users",
    responses(
        (status = 200, description = "Found admins", body = GetAllAdminsResponse),
        (status = 401, description = "Unauthorized", body = JsonError),
        (status = 500, description = "Internal server error occurred", body = JsonError)
    ),
    tag = "Users",
)]
/// Handler for retrieving a list of admin users based on the requester's role permissions.
///
/// This endpoint implements role-based access control to filter which admin records are visible:
/// - **Root admins** see all admins
/// - **Professor admins** see admins with role <= their own
/// - **Tutor admins** see admins with role < their own
/// - **Coordinators** receive 401 Unauthorized (no access)
pub(super) async fn admins_get_all_handler(
    req: HttpRequest, data: Data<AppData>,
) -> Result<HttpResponse, Error> {
    let admin = match req.extensions().get_admin() {
        Ok(u) => u,
        Err(e) => return Err(e.into()),
    };

    let role = match AdminRole::try_from(admin.admin_role_id) {
        Ok(r) => r,
        Err(_) => return Err(ErrorInternalServerError("wrong admin role id".to_string())),
    };

    let found = match role {
        AdminRole::Root => data.repositories.admins.get_all().await,
        AdminRole::Professor => {
            data.repositories
                .admins
                .get_all_from_filter(admins::Column::AdminRoleId.lte(admin.admin_role_id))
                .await
        }
        AdminRole::Tutor => {
            data.repositories
                .admins
                .get_all_from_filter(admins::Column::AdminRoleId.lt(admin.admin_role_id))
                .await
        }
        AdminRole::Coordinator => {
            return Err(ErrorUnauthorized(
                "coordinators do not have permission to view admins".to_string(),
            ))
        }
    };

    let admins: Vec<AdminResponseScheme> = match found {
        Ok(a) => a.into_iter().map(AdminResponseScheme::from).collect(),
        Err(e) => {
            error!("Unable to retrieve admins from database: {}", e);
            return Err(ErrorInternalServerError(
                "unable to retrieve admins from database".to_json_error(),
            ));
        }
    };

    Ok(HttpResponse::Ok().json(GetAllAdminsResponse { admins }))
}
