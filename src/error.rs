use http::StatusCode;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// Maps the [`hmac::digest::InvalidLength`] error.
    #[error(transparent)]
    InvalidLength(#[from] hmac::digest::InvalidLength),
    /// An expected extension was missing.
    #[error("couldn't extract `{0}`. is `SurfLayer` enabled?")]
    ExtensionNotFound(String),
    /// The token cookie couldn't be found by the name given.
    #[error("couldn't get cookie")]
    NoCookie,
}

impl Error {
    pub(crate) fn make_layer_error<T: Default, E>(
        err: impl std::error::Error,
    ) -> Result<http::Response<T>, E> {
        tracing::error!(err = %err);

        let mut response = http::Response::default();
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

        Ok(response)
    }

    pub(crate) fn make_layer_forbidden<T: Default, E>() -> Result<http::Response<T>, E> {
        let mut response = http::Response::default();
        *response.status_mut() = StatusCode::FORBIDDEN;

        Ok(response)
    }
}
