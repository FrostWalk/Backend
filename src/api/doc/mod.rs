use crate::api::v1::admins::auth::login::__path_admins_login_handler;
use crate::api::v1::admins::users::create::__path_create_admin_handler;
use crate::api::v1::admins::users::delete::__path_delete_admin_handler;
use crate::api::v1::admins::users::me::__path_admins_me_handler;
use crate::api::v1::admins::users::read::__path_admins_get_all_handler;
use crate::api::v1::admins::users::read::__path_admins_get_one_handler;
use crate::api::v1::admins::users::update::__path_update_admin_handler;
use crate::api::v1::students::auth::login::__path_students_login_handler;
use crate::api::v1::students::users::me::__path_students_me_handler;
use utoipa::openapi::Server;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        students_login_handler,
        students_me_handler,
        admins_login_handler,
        admins_get_one_handler,
        admins_get_all_handler,
        admins_me_handler,
        create_admin_handler,
        update_admin_handler,
        delete_admin_handler,
    ),
    tags(
        (name = "Admin authentication", description = "Admin authentication endpoint"),
        (name = "Admin users management", description = "CRUD operations on admins"),
        
        (name = "Student authentication", description = "Student authentication endpoint"),
        (name = "Student users management", description = "CRUD operations on students"),
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
