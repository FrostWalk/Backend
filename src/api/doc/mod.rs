use crate::api::v1::admins::auth::login::__path_admins_login_handler;
use crate::api::v1::admins::projects::create::__path_create_project_handler;
use crate::api::v1::admins::projects::delete::__path_delete_project_handler;
use crate::api::v1::admins::projects::read::__path_get_all_projects_handler;
use crate::api::v1::admins::projects::read::__path_get_one_project_handler;
use crate::api::v1::admins::projects::update::__path_update_project_handler;
use crate::api::v1::admins::security_codes::create::__path_create_code_handler;
use crate::api::v1::admins::security_codes::read::__path_get_all_codes_handler;
use crate::api::v1::admins::users::create::__path_create_admin_handler;
use crate::api::v1::admins::users::delete::__path_delete_admin_handler;
use crate::api::v1::admins::users::me::__path_admins_me_handler;
use crate::api::v1::admins::users::read::__path_get_all_admins_handler;
use crate::api::v1::admins::users::read::__path_get_one_admin_handler;
use crate::api::v1::admins::users::update::__path_update_admin_handler;
use crate::api::v1::students::auth::{
    confirm::__path_confirm_student_handler, login::__path_students_login_handler,
    signup::__path_student_signup_handler,
};
use crate::api::v1::students::groups::{
    check_name::__path_check_name, create::__path_create_group, delete::__path_delete_group,
    members::__path_add_member, members::__path_remove_member, read::__path_get_groups,
};
use crate::api::v1::students::projects::read::__path_get_student_projects;
use crate::api::v1::students::security_codes::validate_code::__path_validate_code;
use crate::api::v1::students::users::me::__path_students_me_handler;
use crate::jwt::auth_middleware::{ADMIN_HEADER_NAME, STUDENT_HEADER_NAME};
use utoipa::openapi::security::SecurityScheme;
use utoipa::openapi::security::{ApiKey, ApiKeyValue};
use utoipa::openapi::{Components, Server};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        students_login_handler,
        confirm_student_handler,
        student_signup_handler,
        students_me_handler,
        admins_login_handler,
        get_one_admin_handler,
        get_all_admins_handler,
        admins_me_handler,
        create_admin_handler,
        update_admin_handler,
        delete_admin_handler,
        create_project_handler,
        get_all_projects_handler,
        update_project_handler,
        get_one_project_handler,
        delete_project_handler,
        get_student_projects,
        create_code_handler,
        get_all_codes_handler,
        create_group,
        get_groups,
        delete_group,
        validate_code,
        check_name,
        add_member,
        remove_member,
    ),
    tags(
        (name = "Admin authentication", description = "Admin authentication endpoint"),
        (name = "Admin users management", description = "CRUD operations on admins"),
        (name = "Student authentication", description = "Student authentication endpoint"),
        (name = "Student users management", description = "CRUD operations on students"),
        (name = "Projects management", description = "CRUD operations on projects"),
        (name = "Security codes management", description = "CRUD operations on security codes"),
        (name = "Groups management", description = "CRUD operations on groups and group members"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Ferris store API",
        version = "0.1.0",
        description = "Backend of Ferris store",
        license(name = "MIT", identifier = "MIT")
    ),
)]
pub(in crate::api) struct ApiDoc;

pub(crate) fn open_api() -> SwaggerUi {
    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Ferris store api v1");
    doc.info.version = String::from("0.1.0");
    doc.servers = Some(vec![Server::new("http://localhost:8080/")]);
    SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", doc)
}

#[derive(Default)]
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components: &mut Components =
            openapi.components.get_or_insert_with(Components::default);

        let admin = ApiKeyValue::with_description(ADMIN_HEADER_NAME, "Admin token authentication");
        components.security_schemes.insert(
            "AdminAuth".to_string(),
            SecurityScheme::ApiKey(ApiKey::Header(admin)),
        );

        let user = ApiKeyValue::with_description(STUDENT_HEADER_NAME, "User token authentication");
        components.security_schemes.insert(
            "UserAuth".to_string(),
            SecurityScheme::ApiKey(ApiKey::Header(user)),
        );
    }
}
