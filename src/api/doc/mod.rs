use crate::api::v1::students::auth::login::__path_students_login_handler;
use crate::api::v1::students::users::me::__path_students_me_handler;
use crate::api::v1::admins::users::me::__path_admins_me_handler;
use utoipa::openapi::Server;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        students_login_handler,
        students_me_handler,
        admins_me_handler,
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints")
    )
)]
pub(in crate::api) struct ApiDoc;

pub(crate) fn open_api() -> SwaggerUi {
    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Ferris store api v1");
    doc.info.version = String::from("0.1.0");
    doc.servers = Some(vec![Server::new("http://localhost:8080/")]);
    SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", doc)
}
