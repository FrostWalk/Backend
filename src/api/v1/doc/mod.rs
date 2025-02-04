use crate::api::v1::auth::login::LoginUserSchema;
use crate::api::v1::auth::login::__path_login_handler;
use crate::api::v1::auth::logout::__path_logout_handler;
use crate::api::v1::users::me::UserProjectsSchema;
use crate::api::v1::users::me::__path_me_handler;
use utoipa::openapi::Server;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        login_handler,
        logout_handler,
        me_handler,
    ),
    components(
        schemas(
            LoginUserSchema,
            UserProjectsSchema,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints")
    )
)]
pub(super) struct ApiDoc;

pub(crate) fn open_api() -> SwaggerUi {
    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Ferris store api v1");
    doc.info.version = String::from("0.1.0");
    doc.servers = Some(vec![Server::new("http://localhost:8080/")]);
    SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", doc)
}
