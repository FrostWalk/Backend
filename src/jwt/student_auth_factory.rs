use crate::jwt::auth_middleware::AuthMiddleware;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{ready, Ready};
use std::rc::Rc;

pub(crate) struct Student {}

impl Student {
    pub(crate) fn require_auth() -> Self {
        Self {}
    }
}

impl<S> Transform<S, ServiceRequest> for Student
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<0, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            require_admin: false,
            service: Rc::new(service),
            authentication_only: false,
            allowed_roles: Rc::new([]),
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
            let response = HttpResponse::Ok().body("student_authenticated");
            futures_util::future::ready(Ok(ServiceResponse::new(
                TestRequest::default().to_srv_request().into_parts().0,
                response,
            )))
        }
    }

    #[actix_web::test]
    async fn test_student_factory_creates_middleware_with_correct_config() {
        let student_factory = Student::require_auth();
        let mock_service = MockService;

        let transform = student_factory.new_transform(mock_service).await.unwrap();

        // Verify the middleware is created with correct configuration
        assert!(!transform.require_admin);
        assert!(!transform.authentication_only);
        assert_eq!(transform.allowed_roles.len(), 0);
    }

    #[actix_web::test]
    async fn test_student_factory_creates_working_middleware() {
        let student_factory = Student::require_auth();
        let mock_service = MockService;

        let middleware = student_factory.new_transform(mock_service).await.unwrap();

        // Test that the middleware can be called (though it will fail without proper setup)
        let req = TestRequest::default().to_srv_request();
        let result = middleware.call(req).await;

        // The middleware should fail because there's no token, but the structure should be correct
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_student_factory_always_returns_same_config() {
        let student_factory1 = Student::require_auth();
        let student_factory2 = Student::require_auth();
        let mock_service = MockService;

        let transform1 = student_factory1.new_transform(mock_service).await.unwrap();
        let mock_service2 = MockService;
        let transform2 = student_factory2.new_transform(mock_service2).await.unwrap();

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
    async fn test_student_factory_creates_middleware_for_students_only() {
        let student_factory = Student::require_auth();
        let mock_service = MockService;

        let transform = student_factory.new_transform(mock_service).await.unwrap();

        // Should be configured for students (not admins)
        assert!(!transform.require_admin);
        // Should not be in authentication_only mode
        assert!(!transform.authentication_only);
        // Should have no role restrictions
        assert_eq!(transform.allowed_roles.len(), 0);
    }
}
