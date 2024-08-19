use std::sync::Arc;

use base64::prelude::*;
use hmac::{Hmac, Mac};
use rand::prelude::*;
use sha2::Sha256;
use tower_cookies::{Cookie, Cookies};

use crate::{error::Error, surf::Config};

/// An extension providing a way to interact with a visitor's
/// CSRF token.
#[derive(Clone)]
pub struct Token {
    pub(crate) config: Arc<Config>,
    pub(crate) cookies: Cookies,
}

impl Token {
    pub(crate) fn create(&self) -> Result<(), Error> {
        let identifier: i128 = thread_rng().gen();
        let token = create_token(&self.config.secret, identifier.to_string())?;

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

    /// Updates the identifier used to sign the token. The value should only be valid for the
    /// duration of the user's authenticated session and should be unique to that session.
    ///
    /// See: [OWASP's CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#employing-hmac-csrf-tokens).
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidLength`]
    pub fn set(&self, identifier: impl Into<String>) -> Result<(), Error> {
        let token = create_token(&self.config.secret, identifier)?;

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

    /// Get the current visitor's token.
    ///
    /// # Errors
    ///
    /// - [`Error::NoCookie`]
    pub fn get(&self) -> Result<String, Error> {
        self.cookies
            .get(&self.config.cookie_name())
            .map(|cookie| cookie.value().to_owned())
            .ok_or(Error::NoCookie)
    }

    /// Reset the token to an identifier generated by [Surf](`crate::Surf`).
    pub fn reset(&self) {
        let cookie = Cookie::build((self.config.cookie_name(), "")).build();

        self.cookies.remove(cookie);
    }
}

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn create_token(secret: &str, identifier: impl Into<String>) -> Result<String, Error> {
    let random = BASE64_STANDARD.encode(get_random_value());
    let message = format!("{}!{}", identifier.into(), random);
    let result = sign_and_encode(secret, &message)?;
    let token = format!("{}.{}", result, message);

    Ok(token)
}

pub(crate) fn validate_token(secret: &str, cookie: &str, token: &str) -> Result<bool, Error> {
    let mut parts = token.splitn(2, '.');
    let received_hmac = parts.next().unwrap_or("");

    let message = parts.next().unwrap_or("");
    let expected_hmac = sign_and_encode(secret, message)?;

    Ok(received_hmac == expected_hmac && cookie == token)
}

#[cfg(not(test))]
fn get_random_value() -> [u8; 64] {
    let mut random = [0u8; 64];
    thread_rng().fill(&mut random);

    random
}

#[cfg(test)]
fn get_random_value() -> [u8; 64] {
    [42u8; 64]
}

fn sign_and_encode(secret: &str, message: &str) -> Result<String, Error> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(message.as_bytes());
    let result = BASE64_STANDARD.encode(mac.finalize().into_bytes());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn create_token() -> Result<()> {
        let token = super::create_token("super-secret", "identifier")?;

        let parts = token.splitn(2, '.').collect::<Vec<&str>>();
        assert_eq!(parts.len(), 2);

        let message = format!("{}!{}", "identifier", BASE64_STANDARD.encode([42u8; 64]));
        assert_eq!(parts[1], message);

        let signature = sign_and_encode("super-secret", &message)?;
        assert_eq!(parts[0], signature);

        Ok(())
    }
}
