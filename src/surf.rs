use futures_util::future::BoxFuture;
use http::{Request, Response};
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower_cookies::{
    cookie::{Expiration, SameSite},
    CookieManager, Cookies,
};
use tower_layer::Layer;
use tower_service::Service;

use crate::{guard::GuardService, Error, Token};

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) secret: String,
    pub(crate) cookie_name: String,
    pub(crate) expires: Expiration,
    pub(crate) field_name: String,
    pub(crate) header_name: String,
    pub(crate) http_only: bool,
    pub(crate) prefix: bool,
    pub(crate) same_site: SameSite,
    pub(crate) secure: bool,
}

impl Config {
    pub(crate) fn cookie_name(&self) -> String {
        if self.prefix {
            format!("__HOST-{}", self.cookie_name)
        } else {
            self.cookie_name.clone()
        }
    }
}

#[derive(Clone)]
pub struct Surf {
    pub(crate) config: Config,
}

impl Surf {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            config: Config {
                secret: secret.into(),
                cookie_name: "csrf_token".into(),
                expires: Expiration::Session,
                field_name: "csrf_token".into(),
                header_name: "X-CSRF-Token".into(),
                http_only: true,
                prefix: true,
                same_site: SameSite::Strict,
                secure: true,
            },
        }
    }

    pub fn cookie_name(mut self, cookie_name: impl Into<String>) -> Self {
        self.config.cookie_name = cookie_name.into();

        self
    }

    pub fn expires(mut self, expires: Expiration) -> Self {
        self.config.expires = expires;

        self
    }

    pub fn field_name(mut self, field_name: impl Into<String>) -> Self {
        self.config.field_name = field_name.into();

        self
    }

    pub fn header_name(mut self, header_name: impl Into<String>) -> Self {
        self.config.header_name = header_name.into();

        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.config.http_only = http_only;

        self
    }

    pub fn prefix(mut self, prefix: bool) -> Self {
        self.config.prefix = prefix;

        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.config.same_site = same_site;

        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.config.secure = secure;

        self
    }
}

impl<S> Layer<S> for Surf {
    type Service = CookieManager<SurfService<GuardService<S>>>;

    fn layer(&self, inner: S) -> Self::Service {
        CookieManager::new(SurfService {
            config: Arc::new(self.config.clone()),
            inner: GuardService::new(inner),
        })
    }
}

#[derive(Clone)]
pub struct SurfService<S> {
    config: Arc<Config>,
    inner: S,
}

impl<S, Q, R> Service<Request<Q>> for SurfService<S>
where
    S: Service<Request<Q>, Response = Response<R>> + Send + 'static,
    S::Future: Send + 'static,
    Q: Send + 'static,
    R: Default + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Q>) -> Self::Future {
        let cookies = match request
            .extensions()
            .get::<Cookies>()
            .ok_or(Error::ExtensionNotFound("Cookies".into()))
        {
            Ok(cookies) => cookies,
            Err(err) => return Box::pin(async move { Error::make_layer_error(err) }),
        };

        let token = Token {
            config: self.config.clone(),
            cookies: cookies.clone(),
        };

        if cookies.get(&self.config.cookie_name()).is_none() {
            if let Err(err) = token.create() {
                return Box::pin(async move { Error::make_layer_error(err) });
            };
        }

        request.extensions_mut().insert(self.config.clone());
        request.extensions_mut().insert(token);

        Box::pin(self.inner.call(request))
    }
}
