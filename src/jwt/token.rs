use actix_web::{error, Error};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Token {
    pub(super) sub: i32,
    pub(super) iat: usize,
    pub(super) adm: bool,
    pub(super) rl: i32,
    pub(super) exp: usize,
}

fn create_token(
    user_id: i32, is_admin: bool, admin_role: i32, secret: &[u8], expires_in_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    if user_id < 1 {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
    let claims: Token = Token {
        sub: user_id,
        rl: admin_role,
        adm: is_admin,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}
#[inline(always)]
pub(crate) fn create_admin_token(
    user_id: i32, admin_role_id: i32, secret: &[u8], expires_in_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(user_id, true, admin_role_id, secret, expires_in_seconds)
}
#[inline(always)]
pub(crate) fn create_student_token(
    user_id: i32, secret: &[u8], expires_in_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(user_id, false, 0, secret, expires_in_seconds)
}

pub(super) fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<Token, Error> {
    let decoded = decode::<Token>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );
    decoded
        .map_err(|_| error::ErrorUnauthorized("Invalid token"))
        .map(|token| token.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_create_admin_token_success() {
        let result = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        );

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());

        // Verify token can be decoded
        let decoded = decode_token(&token, TEST_JWT_SECRET);
        assert!(decoded.is_ok());
        let claims = decoded.unwrap();
        assert_eq!(claims.sub, TEST_ADMIN_ID);
        assert_eq!(claims.rl, TEST_ADMIN_ROLE_ID);
        assert!(claims.adm);
    }

    #[test]
    fn test_create_student_token_success() {
        let result =
            create_student_token(TEST_STUDENT_ID, TEST_JWT_SECRET, TEST_JWT_VALIDITY_SECONDS);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());

        // Verify token can be decoded
        let decoded = decode_token(&token, TEST_JWT_SECRET);
        assert!(decoded.is_ok());
        let claims = decoded.unwrap();
        assert_eq!(claims.sub, TEST_STUDENT_ID);
        assert_eq!(claims.rl, 0);
        assert!(!claims.adm);
    }

    #[test]
    fn test_create_token_invalid_user_id() {
        let result = create_admin_token(
            0, // Invalid user ID
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_create_token_negative_user_id() {
        let result = create_admin_token(
            -1, // Invalid user ID
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_decode_token_success() {
        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let result = decode_token(&token, TEST_JWT_SECRET);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, TEST_ADMIN_ID);
        assert_eq!(claims.rl, TEST_ADMIN_ROLE_ID);
        assert!(claims.adm);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_decode_token_invalid_signature() {
        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let wrong_secret = b"wrong-secret-key-for-jwt-tokens-32-chars";
        let result = decode_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_token_malformed() {
        let malformed_token = "not.a.valid.jwt.token";
        let result = decode_token(malformed_token, TEST_JWT_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_token_empty() {
        let result = decode_token("", TEST_JWT_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiration_calculation() {
        let now = Utc::now();
        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            60, // 1 minute
        )
        .unwrap();

        let claims = decode_token(&token, TEST_JWT_SECRET).unwrap();

        // Check that expiration is approximately 1 minute from now
        let expected_exp = (now + Duration::minutes(60)).timestamp() as usize;
        let actual_exp = claims.exp;

        // Allow 5 seconds tolerance for test execution time
        assert!((actual_exp as i64 - expected_exp as i64).abs() <= 5);
    }

    #[test]
    fn test_token_iat_calculation() {
        let before_creation = Utc::now().timestamp() as usize;
        let token = create_admin_token(
            TEST_ADMIN_ID,
            TEST_ADMIN_ROLE_ID,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();
        let after_creation = Utc::now().timestamp() as usize;

        let claims = decode_token(&token, TEST_JWT_SECRET).unwrap();

        // IAT should be between before and after creation
        assert!(claims.iat >= before_creation);
        assert!(claims.iat <= after_creation);
    }

    #[test]
    fn test_student_token_has_zero_role() {
        let token =
            create_student_token(TEST_STUDENT_ID, TEST_JWT_SECRET, TEST_JWT_VALIDITY_SECONDS)
                .unwrap();

        let claims = decode_token(&token, TEST_JWT_SECRET).unwrap();
        assert_eq!(claims.rl, 0);
        assert!(!claims.adm);
    }

    #[test]
    fn test_admin_token_has_correct_role() {
        let role_id = 2; // Different role
        let token = create_admin_token(
            TEST_ADMIN_ID,
            role_id,
            TEST_JWT_SECRET,
            TEST_JWT_VALIDITY_SECONDS,
        )
        .unwrap();

        let claims = decode_token(&token, TEST_JWT_SECRET).unwrap();
        assert_eq!(claims.rl, role_id);
        assert!(claims.adm);
    }
}
