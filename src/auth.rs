use axum::{extract::FromRequestParts, http::StatusCode};
use http::request::Parts;
use serde::Deserialize;
use std::future::{ready, Future};

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub read_only: Option<bool>,
}

pub struct WriteUser;

impl WriteUser {
    fn from_parts_sync(parts: &mut Parts) -> Result<Self, (StatusCode, String)> {
        if let Some(auth_header) = parts.headers.get("x-verified-user-data") {
            let auth_str = auth_header.to_str().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid x-verified-user-data header".to_string(),
                )
            })?;

            let user_data: UserData = serde_json::from_str(auth_str).map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid JSON in x-verified-user-data header".to_string(),
                )
            })?;

            if user_data.read_only.unwrap_or(false) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Read-only access: Cannot modify artifacts".to_string(),
                ));
            }

            Ok(Self)
        } else {
            Err((
                StatusCode::UNAUTHORIZED,
                "Missing x-verified-user-data header".to_string(),
            ))
        }
    }
}

impl<T> FromRequestParts<T> for WriteUser {
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _body: &T,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        ready(WriteUser::from_parts_sync(parts))
    }
}

pub struct VerifiedPath(pub String);

impl<T> FromRequestParts<T> for VerifiedPath {
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _body: &T,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        let header = if let Some(header) = parts.headers.get("x-verified-path") {
            header.to_str().unwrap_or_else(|_| "/").to_string()
        } else {
            "/".to_string()
        };

        std::future::ready(Ok(Self(header)))
    }
}
