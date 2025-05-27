use crate::database::repositories::admins_repository::AdminRole;
use actix_web::{error, Error};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
    user_id: i32, admin_role: AdminRole, secret: &[u8], expires_in_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_token(user_id, true, admin_role.into(), secret, expires_in_seconds)
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
