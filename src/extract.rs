use async_trait::async_trait;
use axum_core::extract::FromRequestParts;
use http::{request::Parts, StatusCode};

use crate::{Error, Token};

#[async_trait]
impl<S> FromRequestParts<S> for Token
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Token>()
            .cloned()
            .ok_or(Error::ExtensionNotFound("Token".into()))
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    }
}
