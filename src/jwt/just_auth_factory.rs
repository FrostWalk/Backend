use crate::jwt::auth_middleware::AuthMiddleware;
use crate::models::admin_role::ALL;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

pub(crate) struct User {}

impl User {
    /// Define a middleware that checks if the user is authenticated regardless of whether it is admin or student
    pub(crate) fn require_auth() -> Self {
        Self {}
    }
}

// implement Transform *only* for the 4-case
impl<S> Transform<S, ServiceRequest> for User
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<3, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            require_admin: false,
            service: Rc::new(service),
            authentication_only: true,
            allowed_roles: Rc::new(ALL),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::admin_role::{AvailableAdminRole, ALL};
    use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
    use actix_web::test::TestRequest;
    use actix_web::HttpResponse;
    use futures_util::future::Ready;
    use std::task::{Context, Poll};

    // Mock service for testing
    struct MockService;
    impl Service<ServiceRequest> for MockService {
        type Response = ServiceResponse<actix_web::body::BoxBody>;
        type Error = actix_web::Error;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&self, _req: ServiceRequest) -> Self::Future {
            let response = HttpResponse::Ok().body("user_authenticated");
            futures_util::future::ready(Ok(ServiceResponse::new(
                TestRequest::default().to_srv_request().into_parts().0,
                response,
            )))
        }
    }

    #[actix_web::test]
    async fn test_user_factory_creates_middleware_with_correct_config() {
        let user_factory = User::require_auth();
        let mock_service = MockService;

        let transform = user_factory.new_transform(mock_service).await.unwrap();

        // Verify the middleware is created with correct configuration
        assert!(!transform.require_admin);
        assert!(transform.authentication_only);
        assert_eq!(transform.allowed_roles.len(), 3);
        assert_eq!(transform.allowed_roles.len(), ALL.len());
    }

    #[actix_web::test]
    async fn test_user_factory_creates_working_middleware() {
        let user_factory = User::require_auth();
        let mock_service = MockService;

        let middleware = user_factory.new_transform(mock_service).await.unwrap();

        // Test that the middleware can be called (though it will fail without proper setup)
        let req = TestRequest::default().to_srv_request();
        let result = middleware.call(req).await;

        // The middleware should fail because there's no token, but the structure should be correct
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_user_factory_always_returns_same_config() {
        let user_factory1 = User::require_auth();
        let user_factory2 = User::require_auth();
        let mock_service = MockService;

        let transform1 = user_factory1.new_transform(mock_service).await.unwrap();
        let mock_service2 = MockService;
        let transform2 = user_factory2.new_transform(mock_service2).await.unwrap();

        // Both should have the same configuration
        assert_eq!(transform1.require_admin, transform2.require_admin);
        assert_eq!(
            transform1.authentication_only,
            transform2.authentication_only
        );
        assert_eq!(
            transform1.allowed_roles.len(),
            transform2.allowed_roles.len()
        );
    }

    #[actix_web::test]
    async fn test_user_factory_creates_middleware_for_any_authenticated_user() {
        let user_factory = User::require_auth();
        let mock_service = MockService;

        let transform = user_factory.new_transform(mock_service).await.unwrap();

        // Should be in authentication_only mode (accepts both admin and student tokens)
        assert!(transform.authentication_only);
        // Should not require admin specifically
        assert!(!transform.require_admin);
        // Should have all admin roles available
        assert_eq!(transform.allowed_roles.len(), 3);
        assert!(transform.allowed_roles.contains(&AvailableAdminRole::Root));
        assert!(transform
            .allowed_roles
            .contains(&AvailableAdminRole::Professor));
        assert!(transform
            .allowed_roles
            .contains(&AvailableAdminRole::Coordinator));
    }

    #[actix_web::test]
    async fn test_user_factory_uses_all_admin_roles() {
        let user_factory = User::require_auth();
        let mock_service = MockService;

        let transform = user_factory.new_transform(mock_service).await.unwrap();

        // Should use the ALL constant which contains all admin roles
        assert_eq!(transform.allowed_roles.len(), ALL.len());
        assert_eq!(transform.allowed_roles.len(), ALL.len());
    }
}
