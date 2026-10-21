use std::{fmt, future::{ready, Ready}};
use actix_web::{dev::Payload, error::Error, FromRequest, HttpRequest, HttpResponse, ResponseError};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::types::user::Claims;

pub const JWT_SECRET: &str = "secret";
#[derive(Debug)] pub struct AuthError;
impl fmt::Display for AuthError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Invalid or missing token") } }
impl ResponseError for AuthError { fn error_response(&self) -> HttpResponse { HttpResponse::BadRequest().json(serde_json::json!({ "message": "Invalid or missing token" })) } }
pub struct AuthUser(pub u32);
impl FromRequest for AuthUser {
    type Error = Error; type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let Some(value) = req.headers().get("Authorization").and_then(|v| v.to_str().ok()) else { return ready(Err(AuthError.into())); };
        let token = value.strip_prefix("Bearer ").unwrap_or(value).trim();
        match decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET.as_bytes()), &Validation::default()) { Ok(data) => ready(Ok(AuthUser(data.claims.sub))), Err(_) => ready(Err(AuthError.into())) }
    }
}
