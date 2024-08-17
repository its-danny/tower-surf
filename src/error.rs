use http::StatusCode;

#[derive(Debug)]
pub enum Error {
    InvalidLength,
    CookiesNotFound,
    ConfigNotFound,
    TokenNotFound,
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

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength => write!(f, "invalid secret length."),
            Error::CookiesNotFound => {
                write!(
                    f,
                    "couldn't extract `Cookies`. is `CookieManagerLayer` enabled?"
                )
            }
            Error::ConfigNotFound => {
                write!(f, "couldn't extract `Config`. is `SurfLayer` enabled?")
            }
            Error::TokenNotFound => {
                write!(f, "couldn't extract `SurfToken`. is `SurfLayer` enabled?")
            }
            Error::NoCookie => write!(f, "couldn't find cookie."),
        }
    }
}
