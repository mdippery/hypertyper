// SPDX-License-Identifier: MIT
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Services for communicating with APIs using HTTP.
// TODO: Expand library documentation,
// including reason for this crate's existence, and common use cases.

use reqwest::{self, header};
use thiserror::Error;

/// An HTTP client provided by an [HTTP service](HTTPService).
pub type Client = reqwest::Client;

/// A general service for making HTTP calls.
///
/// It might be a bit odd to refer to this trait as a "service", since
/// it appears to be more of a _client_ implementation, but think of
/// this as a proxy for a remote _service_ (even though a _client_ is used
/// to communicate with that remote service). A service might not always
/// be remote, such as when the implementation is a deterministic service
/// used for testing.
pub trait HTTPService {
    /// Default HTTP client that can be used to make HTTP requests.
    fn client() -> Client {
        reqwest::ClientBuilder::new()
            .user_agent(Self::user_agent())
            .build()
            // Better error handling? According to the docs, build() only
            // fails if a TLS backend cannot be initialized, or if DNS
            // resolution cannot be initialized, and both of these seem
            // like unrecoverable errors for us.
            .expect("could not create a new HTTP client")
    }

    /// An appropriate user agent to use when making HTTP requests.
    ///
    /// While hypertyper provides a default implementation, you will often
    /// want to use your own. Often times, it is easiest to implement this
    /// as
    ///
    /// ```text
    /// format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    /// ```
    ///
    /// in your own code, like so:
    ///
    /// ```
    /// # use hypertyper::HTTPService;
    /// pub struct MyHttpService;
    ///
    /// impl HTTPService for MyHttpService {
    ///     fn user_agent() -> String {
    ///         format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    ///     }
    /// }
    /// ```
    fn user_agent() -> String {
        // TODO: Consumers will use "hypertyper" as user agent by default.
        // Maybe turn this into a derive macro or something.
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
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
    use crate::HTTPService;
    use regex::Regex;

    #[allow(dead_code)]
    struct UserAgentTestService {}
    impl HTTPService for UserAgentTestService {}

    #[test]
    fn it_returns_user_agent_with_version_number() {
        let user_agent = UserAgentTestService::user_agent();
        let version_re = Regex::new(r"^[a-z]+ v\d+\.\d+\.\d+(-(alpha|beta)(\.\d+)?)?$").unwrap();
        assert!(
            version_re.is_match(&user_agent),
            "{} does not match {}",
            user_agent,
            version_re,
        );
    }
}
