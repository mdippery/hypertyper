// SPDX-License-Identifier: MIT
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! HTTP authentication.

use std::env;

/// Manages authentication keys for HTTP client authorization.
///
/// # Examples
///
/// ```
/// # use hypertyper::auth::Auth;
/// let auth = Auth::new("ThisIsMyApiKey");
/// assert_eq!(auth.api_key(), "ThisIsMyApiKey");
/// ```
#[derive(Debug)]
pub struct Auth {
    api_key: String,
}

impl Auth {
    /// Creates a new `Auth` structure using the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        Self { api_key }
    }

    /// Retrieves an API key from the environment.
    ///
    /// Returns an error if the API key cannot be retrieved from the
    /// environment.
    pub fn from_env(envvar: impl Into<String>) -> Result<Auth, env::VarError> {
        env::var(envvar.into()).map(|api_key| Self { api_key })
    }

    /// The actual API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use temp_env::{with_var, with_var_unset};

    #[test]
    fn it_creates_an_auth_key_from_the_environment() {
        let key_name = "AUTH_API_KEY";
        let key_value = "ThisIsMyApiKey";
        with_var(key_name, Some(key_value), || {
            let auth = Auth::from_env(key_name);
            assert!(auth.is_ok());
            let auth = auth.unwrap();
            assert_eq!(auth.api_key(), key_value);
        })
    }

    #[test]
    fn it_returns_an_error_if_a_key_is_not_set_in_environment() {
        let key_name = "AUTH_API_KEY";
        with_var_unset(key_name, || {
            let auth = Auth::from_env(key_name);
            assert!(auth.is_err());
            assert!(matches!(auth.unwrap_err(), env::VarError::NotPresent))
        });
    }

    #[test]
    fn it_returns_an_error_if_a_key_is_not_unicode() {
        let key_name = "AUTH_API_KEY";
        let bytes = vec![0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        let key_value = unsafe { OsString::from_encoded_bytes_unchecked(bytes) };
        with_var(key_name, Some(key_value), || {
            let auth = Auth::from_env(key_name);
            assert!(auth.is_err());
            assert!(matches!(auth.unwrap_err(), env::VarError::NotUnicode(_)))
        })
    }
}
