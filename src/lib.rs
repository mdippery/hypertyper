// SPDX-License-Identifier: MIT
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Useful types and idioms (and a few implementations) for building HTTP/S clients.
//!
//! **Hypertyper** provides convenient ways to build and use HTTP/S clients.
//! Configure an [`HTTPClientFactory`] once and use it to produce as many
//! [`HTTPClient`] instances as needed. Use [`HTTPResult`] to provide a
//! common way to return HTTP response bodies or errors, and wrap HTTP errors
//! in a common [`HTTPError`] enum to unify your HTTP response handling.
//!
//! Under the hood, Hypertyper uses the excellent [reqwest] library to
//! satisfy all your HTTP needs.
//!
//! # Usage
//!
//! With [`HTTPClientFactory`], you can [configure a factory once] and use it to
//! produce identical HTTP clients as needed. For example, you configure set a
//! [user agent] when you create the factory, and any HTTP clients created by
//! that factory will automatically use that user agent string when making HTTP
//! calls.
//!
//! You can also define your own calls to return a common [`HTTPResult`], and
//! wrap errors using the [`HTTPError`] enum.
//!
//! # History
//!
//! Hypertyper was created to wrap the most common HTTP-related code into a
//! common interface usable across libraries and applications. It is a
//! rapidly-evolving project that will grow to encapsulate the most common
//! HTTP types, idioms, and operations, allowing you to focus on the specific
//! needs of your applications.
//!
//! [reqwest]: https://crates.io/crates/reqwest
//! [configure a factory once]: HTTPClientFactory::with_user_agent()
//! [user agent]: HTTPClientFactory::user_agent()

pub mod auth;
pub mod service;

pub use crate::auth::Auth;
pub use crate::service::HTTPService;
pub use reqwest::IntoUrl;
use reqwest::{self, header};
use thiserror::Error;

/// An HTTP client created by an [`HTTPClientFactory`].
///
/// This is identical to a [`reqwest::Client`], but that could change in
/// the future, so consumers of this crate are encouraged to use this
/// type alias instead of referencing `reqwest::Client` directly.
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
pub type HTTPClient = reqwest::Client;

/// Produces new HTTP clients from a template.
///
/// For example, this makes it easy to create new clients with a standard
/// user agent.
///
/// Most commonly, you will call [`HTTPClientFactory::new()`] with your package
/// name and version to construct a standardized user agent string based on
/// your package, but you can also call [`HTTPClientFactory::with_user_agent()`]
/// to supply your own custom user agent string.
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
        let user_agent = format!("{} v{}", pkg_name.into(), pkg_version.into());
        HTTPClientFactory::with_user_agent(user_agent)
    }

    /// Create a new factory that will produce clients with the given user agent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hypertyper::HTTPClientFactory;
    /// let factory = HTTPClientFactory::with_user_agent("my cool user agent");
    /// assert_eq!(factory.user_agent(), "my cool user agent");
    /// ```
    pub fn with_user_agent(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
        }
    }

    /// Creates a new client that can be used to make HTTP requests.
    ///
    /// # Panics
    ///
    /// This method panics if a TLS backend cannot be initialized.
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
///
/// Often times, the type argument `T` is either a `String`, or a type that
/// can be deserialized with [serde_json].
///
/// [serde_json]: https://crates.io/crates/serde_json
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
