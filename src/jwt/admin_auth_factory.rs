use crate::jwt::auth_middleware::AuthMiddleware;
use crate::models::admin_role::AvailableAdminRole;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

pub(crate) struct Admin<const N: usize> {
    allowed_roles: Rc<[AvailableAdminRole; N]>,
}

impl<const N: usize> Admin<N> {
    pub(crate) fn require_roles(allowed_roles: [AvailableAdminRole; N]) -> Self {
        Self {
            allowed_roles: Rc::new(allowed_roles),
        }
    }
}

impl<const N: usize, S> Transform<S, ServiceRequest> for Admin<N>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<N, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            require_admin: true,
            service: Rc::new(service),
            authentication_only: false,
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let response = HttpResponse::Ok().body("admin_authenticated");
            futures_util::future::ready(Ok(ServiceResponse::new(
                TestRequest::default().to_srv_request().into_parts().0,
                response,
            )))
        }
    }

    #[actix_web::test]
    async fn test_admin_factory_creates_middleware_with_correct_config() {
        let admin_factory =
            Admin::require_roles([AvailableAdminRole::Root, AvailableAdminRole::Professor]);
        let mock_service = MockService;

        let transform = admin_factory.new_transform(mock_service).await.unwrap();

        // Verify the middleware is created with correct configuration
        assert!(transform.require_admin);
        assert!(!transform.authentication_only);
        assert_eq!(transform.allowed_roles.len(), 2);
        assert!(transform.allowed_roles.contains(&AvailableAdminRole::Root));
        assert!(transform
            .allowed_roles
            .contains(&AvailableAdminRole::Professor));
    }

    #[actix_web::test]
    async fn test_admin_factory_with_single_role() {
        let admin_factory = Admin::require_roles([AvailableAdminRole::Root]);
        let mock_service = MockService;

        let transform = admin_factory.new_transform(mock_service).await.unwrap();

        assert!(transform.require_admin);
        assert!(!transform.authentication_only);
        assert_eq!(transform.allowed_roles.len(), 1);
        assert!(transform.allowed_roles.contains(&AvailableAdminRole::Root));
    }

    #[actix_web::test]
    async fn test_admin_factory_with_empty_roles() {
        let admin_factory = Admin::require_roles([]);
        let mock_service = MockService;

        let transform = admin_factory.new_transform(mock_service).await.unwrap();

        assert!(transform.require_admin);
        assert!(!transform.authentication_only);
        assert_eq!(transform.allowed_roles.len(), 0);
    }

    #[actix_web::test]
    async fn test_admin_factory_with_multiple_roles() {
        let admin_factory = Admin::require_roles([
            AvailableAdminRole::Root,
            AvailableAdminRole::Professor,
            AvailableAdminRole::Coordinator,
        ]);
        let mock_service = MockService;

        let transform = admin_factory.new_transform(mock_service).await.unwrap();

        assert!(transform.require_admin);
        assert!(!transform.authentication_only);
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
    async fn test_admin_factory_creates_working_middleware() {
        let admin_factory = Admin::require_roles([AvailableAdminRole::Root]);
        let mock_service = MockService;

        let middleware = admin_factory.new_transform(mock_service).await.unwrap();

        // Test that the middleware can be called (though it will fail without proper setup)
        let req = TestRequest::default().to_srv_request();
        let result = middleware.call(req).await;

        // The middleware should fail because there's no token, but the structure should be correct
        assert!(result.is_err());
    }
}
