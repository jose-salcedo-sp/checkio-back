use axum::{extract::FromRequestParts, http::request::Parts};
use crate::{auth::verify_token, error::AppError};

pub struct AuthUser {
    pub user_id: String,
    pub role:    String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_token(token, &jwt_secret)?.claims;

        Ok(AuthUser {
            user_id: claims.sub,
            role:    claims.role,
        })
    }
}