use std::sync::Arc;

use base64::prelude::*;
use hmac::Mac;
use rand::prelude::*;
use tower_cookies::{Cookie, Cookies};

use crate::{error::Error, surf::Config, HmacSha256};

#[derive(Clone)]
pub struct Token {
    pub(crate) config: Arc<Config>,
    pub(crate) cookies: Cookies,
}

impl Token {
    pub(crate) fn create(&self) -> Result<(), Error> {
        let identifier: i128 = thread_rng().gen();
        let token = self.sign(identifier.to_string())?;

        let cookie = Cookie::build((self.config.cookie_name(), token))
            .path("/")
            .expires(self.config.expires)
            .http_only(self.config.http_only)
            .same_site(self.config.same_site)
            .secure(self.config.secure)
            .build();

        self.cookies.add(cookie);

        Ok(())
    }

    pub fn set(&self, identifier: impl Into<String>) -> Result<(), Error> {
        let token = self.sign(identifier)?;

        let cookie = Cookie::build((self.config.cookie_name(), token))
            .path("/")
            .expires(self.config.expires)
            .http_only(self.config.http_only)
            .same_site(self.config.same_site)
            .secure(self.config.secure)
            .build();

        self.cookies.add(cookie);

        Ok(())
    }

    pub fn get(&self) -> Result<String, Error> {
        self.cookies
            .get(&self.config.cookie_name())
            .map(|cookie| cookie.value().to_owned())
            .ok_or(Error::NoCookie)
    }

    pub fn reset(&self) {
        let cookie = Cookie::build((self.config.cookie_name(), "")).build();

        self.cookies.remove(cookie);
    }

    fn sign(&self, identifier: impl Into<String>) -> Result<String, Error> {
        let mut random = [0u8; 64];
        thread_rng().fill(&mut random);
        let random = BASE64_STANDARD.encode(random);

        let message = format!("{}!{}", identifier.into(), random);
        let mut mac = HmacSha256::new_from_slice(self.config.secret.as_bytes())?;
        mac.update(message.as_bytes());
        let result = BASE64_STANDARD.encode(mac.finalize().into_bytes());

        let token = format!("{}.{}", result, message);

        Ok(token)
    }
}
