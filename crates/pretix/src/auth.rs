use std::fmt;

use http::HeaderValue;
use secrecy::{ExposeSecret, SecretString};

use crate::Error;

/// Credentials used to authenticate against the pretix API.
///
/// Only team API tokens are supported for now. The type is opaque so that further
/// schemes (OAuth, device authentication) can be added without breaking changes.
#[derive(Clone)]
pub struct Auth(Kind);

#[derive(Clone)]
enum Kind {
    Token(SecretString),
}

impl Auth {
    /// Team API token authentication (`Authorization: Token <token>`).
    pub fn token(token: impl Into<String>) -> Self {
        Self(Kind::Token(SecretString::from(token.into())))
    }

    pub(crate) fn header_value(&self) -> Result<HeaderValue, Error> {
        let raw = match &self.0 {
            Kind::Token(token) => format!("Token {}", token.expose_secret()),
        };
        let mut value = HeaderValue::try_from(raw)
            .map_err(|_| Error::Config("the API token contains invalid characters".into()))?;
        value.set_sensitive(true);
        Ok(value)
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Kind::Token(_) => f.write_str("Auth::Token(<redacted>)"),
        }
    }
}
