use crate::app_data::AppData;
use actix_web::web::Data;
use actix_web::{Error, HttpRequest, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateAdminResponse {}

pub(super) async fn create_admin_handler(
    req: HttpRequest, data: Data<AppData>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(""))
}
