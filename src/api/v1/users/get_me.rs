use crate::common::json_error::JsonError;
use actix_web::error::ErrorNotFound;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use entity::users::Model;
use serde_json::json;

pub(super) async fn get_me(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Retrieve the user information from the request extensions.
    match req.extensions().get::<Model>() {
        Some(user) => {
            // Filter sensitive user data before sending the response.

            // Prepare the response data with the filtered user information.
            let response_data = json!({
                "user": user
            });

            // Respond with the filtered user information in JSON format.
            Ok(HttpResponse::Ok().json(response_data))
        }
        None => Err(ErrorNotFound("user does not exists".to_json_error())),
    }
}
