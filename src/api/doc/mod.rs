use crate::api::health::{__path_health_check, __path_liveness_check};
use crate::api::v1::admins::auth::forgot_password::__path_forgot_password_handler;
use crate::api::v1::admins::auth::login::__path_admins_login_handler;
use crate::api::v1::admins::auth::reset_password::__path_reset_password_handler;
use crate::api::v1::admins::group_deliverable_components::create::__path_create_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::delete::__path_delete_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::read::__path_get_all_group_components_handler;
use crate::api::v1::admins::group_deliverable_components::read::__path_get_deliverables_for_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::read::__path_get_group_component_handler;
use crate::api::v1::admins::group_deliverable_components::read::__path_get_group_components_for_project_handler;
use crate::api::v1::admins::group_deliverable_components::update::__path_update_group_component_handler;
use crate::api::v1::admins::group_deliverable_selections::read::__path_get_group_deliverable_selections;
use crate::api::v1::admins::group_deliverables::create::__path_create_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::delete::__path_delete_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::read::__path_get_all_group_deliverables_handler;
use crate::api::v1::admins::group_deliverables::read::__path_get_components_for_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::read::__path_get_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables::read::__path_get_group_deliverables_for_project_handler;
use crate::api::v1::admins::group_deliverables::update::__path_update_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables_and_components::create::__path_create_group_deliverable_component_handler;
use crate::api::v1::admins::group_deliverables_and_components::delete::__path_delete_group_deliverable_component_handler;
use crate::api::v1::admins::group_deliverables_and_components::read::__path_get_components_for_deliverable_handler as __path_get_group_components_for_group_deliverable_handler;
use crate::api::v1::admins::group_deliverables_and_components::read::__path_get_deliverables_for_component_handler as __path_get_group_deliverables_for_group_component_handler;
use crate::api::v1::admins::group_deliverables_and_components::update::__path_update_group_deliverable_component_handler;
use crate::api::v1::admins::groups::details::__path_get_group_details;
use crate::api::v1::admins::groups::members::{
    __path_add_member as __path_admin_add_member,
    __path_remove_member as __path_admin_remove_member, __path_transfer_leadership,
};
use crate::api::v1::admins::groups::read::__path_get_project_groups;
use crate::api::v1::admins::projects::coordinators::{
    __path_assign_coordinator, __path_list_coordinators, __path_remove_coordinator,
};
use crate::api::v1::admins::projects::create::__path_create_project_handler;
use crate::api::v1::admins::projects::delete::__path_delete_project_handler;
use crate::api::v1::admins::projects::read::__path_get_all_projects_handler;
use crate::api::v1::admins::projects::read::__path_get_one_project_handler;
use crate::api::v1::admins::projects::update::__path_update_project_handler;
use crate::api::v1::admins::security_codes::create::__path_create_code_handler;
use crate::api::v1::admins::security_codes::read::__path_get_all_codes_handler;
use crate::api::v1::admins::student_deliverable_components::create::__path_create_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::delete::__path_delete_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::read::__path_get_all_student_components_handler;
use crate::api::v1::admins::student_deliverable_components::read::__path_get_deliverables_for_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::read::__path_get_student_component_handler;
use crate::api::v1::admins::student_deliverable_components::read::__path_get_student_components_for_project_handler;
use crate::api::v1::admins::student_deliverable_components::update::__path_update_student_component_handler;
use crate::api::v1::admins::student_deliverable_selections::read::__path_get_student_deliverable_selections;
use crate::api::v1::admins::student_deliverables::create::__path_create_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::delete::__path_delete_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::read::__path_get_all_student_deliverables_handler;
use crate::api::v1::admins::student_deliverables::read::__path_get_components_for_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::read::__path_get_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables::read::__path_get_student_deliverables_for_project_handler;
use crate::api::v1::admins::student_deliverables::update::__path_update_student_deliverable_handler;
use crate::api::v1::admins::student_deliverables_and_components::create::__path_create_student_deliverable_component_handler;
use crate::api::v1::admins::student_deliverables_and_components::delete::__path_delete_student_deliverable_component_handler;
use crate::api::v1::admins::student_deliverables_and_components::read::__path_get_components_for_deliverable_handler;
use crate::api::v1::admins::student_deliverables_and_components::read::__path_get_deliverables_for_component_handler;
use crate::api::v1::admins::student_deliverables_and_components::update::__path_update_student_deliverable_component_handler;
use crate::api::v1::admins::users::create::__path_create_admin_handler;
use crate::api::v1::admins::users::delete::__path_delete_admin_handler;
use crate::api::v1::admins::users::me::__path_admins_me_handler;
use crate::api::v1::admins::users::read::__path_get_all_admins_handler;
use crate::api::v1::admins::users::read::__path_get_one_admin_handler;
use crate::api::v1::admins::users::update::__path_update_admin_handler;
use crate::api::v1::admins::users::update_me::__path_update_me_admin_handler;
use crate::api::v1::students::auth::{
    allowed_domains::__path_allowed_domains_handler, confirm::__path_confirm_student_handler,
    forgot_password::__path_forgot_password_handler as __path_students_forgot_password_handler,
    login::__path_students_login_handler,
    reset_password::__path_reset_password_handler as __path_students_reset_password_handler,
    signup::__path_student_signup_handler,
};
use crate::api::v1::students::group_deliverable_selections::{
    create::__path_create_group_deliverable_selection,
    read::__path_get_group_deliverable_selection,
    update::__path_update_group_deliverable_selection,
};
use crate::api::v1::students::groups::{
    check_name::__path_check_name, create::__path_create_group, delete::__path_delete_group,
    members::__path_add_member, members::__path_remove_member,
    members_list::__path_list_group_members, read::__path_get_groups,
};
use crate::api::v1::students::projects::read::__path_get_student_projects;
use crate::api::v1::students::security_codes::validate_code::__path_validate_code;
use crate::api::v1::students::student_deliverable_selections::{
    create::__path_create_student_deliverable_selection,
    delete::__path_delete_student_deliverable_selection,
    read::__path_get_student_deliverable_selection,
    update::__path_update_student_deliverable_selection,
};
use crate::api::v1::students::users::me::__path_students_me_handler;
use crate::api::v1::students::users::update_me::__path_update_me_student_handler;
use crate::api::version::__path_version_info;
use crate::jwt::auth_middleware::{ADMIN_HEADER_NAME, STUDENT_HEADER_NAME};
use utoipa::openapi::security::SecurityScheme;
use utoipa::openapi::security::{ApiKey, ApiKeyValue};
use utoipa::openapi::{Components, Server};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        liveness_check,
        version_info,
        allowed_domains_handler,
        students_login_handler,
        confirm_student_handler,
        student_signup_handler,
        students_forgot_password_handler,
        students_reset_password_handler,
        students_me_handler,
        update_me_student_handler,
        admins_login_handler,
        forgot_password_handler,
        reset_password_handler,
        get_one_admin_handler,
        get_all_admins_handler,
        admins_me_handler,
        update_me_admin_handler,
        create_admin_handler,
        update_admin_handler,
        delete_admin_handler,
        create_project_handler,
        get_all_projects_handler,
        update_project_handler,
        get_one_project_handler,
        delete_project_handler,
        assign_coordinator,
        list_coordinators,
        remove_coordinator,
        get_project_groups,
        get_group_details,
        admin_remove_member,
        transfer_leadership,
        admin_add_member,
        get_group_deliverable_selections,
        get_student_deliverable_selections,
        get_student_projects,
        create_code_handler,
        get_all_codes_handler,
        create_group_component_handler,
        get_all_group_components_handler,
        get_group_component_handler,
        get_group_components_for_project_handler,
        get_deliverables_for_group_component_handler,
        update_group_component_handler,
        delete_group_component_handler,
        create_group_deliverable_handler,
        get_all_group_deliverables_handler,
        get_group_deliverable_handler,
        get_group_deliverables_for_project_handler,
        get_components_for_group_deliverable_handler,
        update_group_deliverable_handler,
        delete_group_deliverable_handler,
        create_group_deliverable_component_handler,
        get_group_components_for_group_deliverable_handler,
        get_group_deliverables_for_group_component_handler,
        update_group_deliverable_component_handler,
        delete_group_deliverable_component_handler,
        create_student_component_handler,
        get_all_student_components_handler,
        get_student_component_handler,
        get_student_components_for_project_handler,
        get_deliverables_for_student_component_handler,
        update_student_component_handler,
        delete_student_component_handler,
        create_student_deliverable_handler,
        get_all_student_deliverables_handler,
        get_student_deliverable_handler,
        get_student_deliverables_for_project_handler,
        get_components_for_student_deliverable_handler,
        update_student_deliverable_handler,
        delete_student_deliverable_handler,
        create_student_deliverable_component_handler,
        get_components_for_deliverable_handler,
        get_deliverables_for_component_handler,
        update_student_deliverable_component_handler,
        delete_student_deliverable_component_handler,
        create_group,
        get_groups,
        delete_group,
        validate_code,
        check_name,
        add_member,
        remove_member,
        list_group_members,
        create_group_deliverable_selection,
        get_group_deliverable_selection,
        update_group_deliverable_selection,
        create_student_deliverable_selection,
        get_student_deliverable_selection,
        update_student_deliverable_selection,
        delete_student_deliverable_selection,
    ),
    tags(
        (name = "Health", description = "Application health check endpoints for monitoring and Docker"),
        (name = "Version", description = "Application version information endpoints"),
        (name = "Admin authentication", description = "Admin authentication endpoint"),
        (name = "Admin users management", description = "CRUD operations on admins"),
        (name = "Group deliverable components management", description = "CRUD operations on group deliverable components"),
        (name = "Group deliverables management", description = "CRUD operations on group deliverables"),
        (name = "Group deliverables-components management", description = "CRUD operations on group deliverables-components relationships"),
        (name = "Student deliverable components management", description = "CRUD operations on student deliverable components"),
        (name = "Student deliverables management", description = "CRUD operations on student deliverables"),
        (name = "Student deliverables-components management", description = "CRUD operations on student deliverables-components relationships"),
        (name = "Student authentication", description = "Student authentication endpoint"),
        (name = "Student users management", description = "CRUD operations on students"),
        (name = "Projects management", description = "CRUD operations on projects"),
        (name = "Security codes management", description = "CRUD operations on security codes"),
        (name = "Groups management", description = "CRUD operations on groups and group members"),
        (name = "Group Deliverable Selections", description = "Operations for group deliverable selections"),
        (name = "Student Deliverable Selections", description = "Operations for student deliverable selections"),
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Advanced Programming Application Backend API",
        version = "0.1.0",
        description = "This is the description of the APIs exposed by the backend of the advanced programming application",
        license(name = "MIT", identifier = "MIT")
    ),
)]
pub(in crate::api) struct ApiDoc;

pub(crate) fn open_api() -> SwaggerUi {
    let mut doc = ApiDoc::openapi();
    doc.info.title = String::from("Advanced Programming Application Backend API v1");
    doc.info.version = String::from("0.1.0");
    doc.servers = Some(vec![
        Server::new("https://dev.advancedprogramming.ovh/api"),
        Server::new("http://localhost:8080/"),
    ]);
    SwaggerUi::new("/swagger/{_:.*}").url("/swagger-openapi.json", doc)
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
            "StudentAuth".to_string(),
            SecurityScheme::ApiKey(ApiKey::Header(user)),
        );
    }
}
