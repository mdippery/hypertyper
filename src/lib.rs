// SPDX-License-Identifier: MIT
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Services for communicating with APIs using HTTP.
// TODO: Expand library documentation,
// including reason for this crate's existence, and common use cases.

use reqwest::{self, header};
use thiserror::Error;

/// An HTTP client provided by an [HTTP service](HTTPService).
pub type HTTPClient = reqwest::Client;

/// Produces new HTTP clients from a template.
///
/// For example, this makes it easy to create new clients with a standard
/// user agent.
#[derive(Debug)]
pub struct HTTPClientFactory {
    user_agent: String,
}

impl HTTPClientFactory {
    /// Create a new factory using the given package name and version as a basis
    /// for the clients' user agents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hypertyper::HTTPClientFactory;
    /// let factory = HTTPClientFactory::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    /// let user_agent = factory.user_agent();
    /// assert!(user_agent.starts_with(env!("CARGO_PKG_NAME")));
    /// assert!(user_agent.ends_with(env!("CARGO_PKG_VERSION")));
    /// ```
    pub fn new(pkg_name: impl Into<String>, pkg_version: impl Into<String>) -> Self {

        // TODO: Could this be a macro that is evaluated at the consumer's compilation time?
        // It would be handy for callers to call something like `user_agent!()` or
        // `default_factory!()` or whatever, which would then construct a user agent
        // using the caller's CARGO_PKG_NAME and CARGO_PKG_VERSION.

        let user_agent = format!("{} v{}", pkg_name.into(), pkg_version.into());
        HTTPClientFactory::new_with_user_agent(user_agent)
    }

    /// Create a new factory that will produce clients with the given user agent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hypertyper::HTTPClientFactory;
    /// let factory = HTTPClientFactory::new_with_user_agent("my cool user agent");
    /// assert_eq!(factory.user_agent(), "my cool user agent");
    /// ```
    pub fn new_with_user_agent(user_agent: impl Into<String>) -> Self {
        Self { user_agent: user_agent.into() }
    }

    /// Creates a new client that can be used to make HTTP requests.
    pub fn create(&self) -> HTTPClient {
        reqwest::ClientBuilder::new()
            .user_agent(self.user_agent())
            .build()
            // Better error handling? According to the docs, build() only
            // fails if a TLS backend cannot be initialized, or if DNS
            // resolution cannot be initialized, and both of these seem
            // like unrecoverable errors for us.
            .expect("could not create a new HTTP client")
    }

    /// The user agent used in HTTP clients produced by this factory.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
}

/// The result of an HTTP request.
pub type HTTPResult<T> = Result<T, HTTPError>;

/// Indicates an error has occurred when making an HTTP call.
#[derive(Debug, Error)]
pub enum HTTPError {
    /// An error that occurred while making an HTTP request.
    #[error("Error while making or processing an HTTP request: {0}")]
    Request(#[from] reqwest::Error),

    /// An error that occurred while trying to serialize a POST body.
    #[error("Error serializing POST body: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An unsuccessful HTTP status code in an HTTP response.
    #[error("Request returned HTTP {0}")]
    Http(reqwest::StatusCode),

    /// A missing Content-Type header in a response.
    #[error("Missing Content-Type header")]
    MissingContentType,

    /// An invalid Content-Type header.
    #[error("Invalid Content-Type header value: {0}")]
    InvalidContentType(#[from] header::ToStrError),

    /// A Content-Type that is not understood by the service.
    #[error("Unexpected content type: {0}")]
    UnexpectedContentType(String),
}

#[cfg(test)]
mod tests {
    use crate::HTTPClientFactory;
    use regex::Regex;

    impl Default for HTTPClientFactory {
        fn default() -> Self {
            let user_agent = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            Self { user_agent }
        }
    }

    #[test]
    fn it_returns_user_agent_with_version_number() {
        let factory = HTTPClientFactory::default();
        let user_agent = factory.user_agent();
        let version_re = Regex::new(r"^[a-z]+ v\d+\.\d+\.\d+(-(alpha|beta)(\.\d+)?)?$").unwrap();
        assert!(
            version_re.is_match(&user_agent),
            "{} does not match {}",
            user_agent,
            version_re,
        );
    }
}
