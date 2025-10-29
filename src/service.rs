// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Traits and structs for communicating with HTTP servers.
//!
//! In this context, a "service" is a mechanism for communicating with a
//! remote HTTP server; for example, it provides a uniform interface for
//! making GET and POST requests and receiving an HTTP response. It might
//! be more reasonable to think of this as an HTTP _client_, but here,
//! we refer to "service" because it is a proxy for a remote HTTP service,
//! and we want to avoid confusion with an _API client_, which will most
//! likely use this "service".
//!
//! HTTP service structs are often created to provide low-level HTTP
//! communication for API clients.
//!
//! # Usage
//!
//! If HTTP service structs are most commonly used by API clients, why use
//! a separate HTTP service struct at all? Could an API client not just,
//! e.g., create its own Reqwest client and call its methods directly?
//!
//! An API client certainly could do so, but that presents problems for
//! testing. It is often useful to provide static responses when testing
//! code that communicates over HTTP, and it is especially useful in unit
//! tests to avoid making external HTTP calls. Thus, it is often useful for
//! API clients to take a generic HTTP service as an argument; a test
//! client that returns static responses can be implemented and passed to
//! the API client during testing.
//!
//! To simplify the API clients' interfaces, a default constructor can
//! create a "real" HTTP service, and a private method that is only
//! available for internal tests can take an HTTP service instance.
//!
//! ```
//! use hypertyper::{
//!     HTTPClient,
//!     HTTPClientFactory,
//!     HTTPError,
//!     HTTPGet,
//!     HTTPPost,
//!     HTTPResult,
//!     IntoUrl
//! };
//! use hypertyper::auth::Auth;
//! use hypertyper::service::HTTPService;
//! use reqwest::{header, StatusCode};
//! use serde::{Serialize, de::DeserializeOwned};
//! use std::fs;
//!
//! pub struct RealService {
//!   auth: Auth,
//!   client: HTTPClient,
//! }
//!
//! impl RealService {
//!     pub fn new(auth: Auth, factory: HTTPClientFactory) -> Self {
//!       let client = factory.create();
//!       Self { auth, client }
//!     }
//! }
//!
//! impl HTTPGet for RealService {
//!     async fn get<U>(&self, uri: U) -> HTTPResult<String>
//!     where
//!         U: IntoUrl + Send
//!     {
//!         Ok(self.client.get(uri).send().await?.text().await?)
//!     }
//! }
//!
//! impl HTTPPost for RealService {
//!     async fn post<U, D, R>(&self, uri: U, auth: &Auth, data: &D) -> HTTPResult<R>
//!     where
//!         U: IntoUrl + Send,
//!         D: Serialize + Sync,
//!         R: DeserializeOwned,
//!     {
//!         let json_object = self
//!             .client
//!             .post(uri)
//!             .header(header::CONTENT_TYPE, "application/json")
//!             .json(data)
//!             .send()
//!             .await?
//!             .json::<R>()
//!             .await?;
//!         Ok(json_object)
//!     }
//! }
//!
//! #[derive(Default)]
//! pub struct TestService;
//!
//! impl HTTPGet for TestService {
//!     async fn get<U>(&self, uri: U) -> HTTPResult<String>
//!     where
//!         U: IntoUrl + Send
//!     {
//!         let path = format!("tests/data{}", uri.as_str());
//!         Ok(fs::read_to_string(path).expect("could not find test data"))
//!     }
//! }
//!
//! impl HTTPPost for TestService {
//!     async fn post<U, D, R>(&self, uri: U, auth: &Auth, data: &D) -> HTTPResult<R>
//!     where
//!         U: IntoUrl + Send,
//!         D: Serialize + Sync,
//!         R: DeserializeOwned,
//!     {
//!         Err(HTTPError::Http(StatusCode::INTERNAL_SERVER_ERROR))
//!     }
//! }
//!
//! pub struct APIClient<S: HTTPService> {
//!     service: S,
//! }
//!
//! impl<S: HTTPService> APIClient<S> {
//!     // Note: Not pub! This will only be available in tests within the module.
//!     // Use pub(crate) fn if with_service() should be available in test utils
//!     // modules, too.
//!     fn with_service(service: S) -> Self {
//!         Self { service }
//!     }
//! }
//!
//! impl APIClient<RealService> {
//!     pub fn new(auth: Auth) -> Self {
//!         let factory = HTTPClientFactory::with_user_agent("my cool user agent");
//!         let service = RealService::new(auth, factory);
//!         Self::with_service(service)
//!     }
//! }
//!
//! let auth = Auth::new("some-cool-api-key");
//! let real_client = APIClient::new(auth);
//!
//! // APIClient::with_service() is only available within the module,
//! // which simplifies the public API while allowing easy testing.
//! let test_client = APIClient::with_service(TestService::default());
//! ```
//!
//! Together, an HTTP service trait and its various concrete implementations
//! provide a uniform way of communicating over HTTP, whether code is
//! under test or live in production.

#[cfg(feature = "test-utils")]
pub mod testing;

use crate::{Auth, HTTPResult, IntoUrl};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// An [HTTP service](HTTPService) that only makes HTTP GET requests.
pub trait HTTPGet {
    /// Performs a GET request to the given URI and returns the raw body.
    ///
    /// # Examples
    ///
    /// The simplest implementation of this method is
    ///
    /// ```text
    /// Ok(self.client.get(uri).send().await?.text().await?)
    /// ```
    ///
    /// (where `self.client` is a [Reqwest client]).
    ///
    /// [Reqwest client]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    fn get<U>(&self, uri: U) -> impl Future<Output = HTTPResult<String>> + Send
    where
        U: IntoUrl + Send;
}

/// An [HTTP service](HTTPService) that only makes HTTP POST requests.
pub trait HTTPPost {
    /// Send a POST request to the `uri` with the JSON object `data` as
    /// the POST request body.
    ///
    /// The response is deserialized from a string to the JSON object
    /// specified by the `R` type parameter.
    ///
    /// # Examples
    ///
    /// A simple implementation of this method with bearer authentication is
    ///
    /// ```text
    /// // use reqwest::header;
    ///
    /// let auth_header = format!("Bearer {}", auth.api_key());
    /// let json_object = self
    ///     .client
    ///     .post(uri)
    ///     .header(header::CONTENT_TYPE, "application/json")
    ///     .header(header::AUTHORIZATION, auth_header)
    ///     .json(data)
    ///     .send()
    ///     .await?
    ///     .json::<R>()
    ///     .await?;
    /// Ok(json_object)
    /// ```
    ///
    /// (where `self.client` is a [Reqwest client] and `auth` is an [`Auth`] instance).
    ///
    /// [Reqwest client]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
    fn post<U, D, R>(
        &self,
        uri: U,
        auth: &Auth, // TODO: Auth should be optional, or specified in an auth() method (builder pattern?)
        data: &D,
    ) -> impl Future<Output = HTTPResult<R>> + Send
    where
        U: IntoUrl + Send,
        D: Serialize + Sync,
        R: DeserializeOwned;
}

/// A service for making calls to an HTTP server and handling responses.
///
/// # Usage
///
/// Using this trait, clients can implement different ways of connecting
/// to an HTTP server, such as an actual connector for production code,
/// and a mocked connector for testing purposes.
///
/// See the [module documentation] for examples on how to use this trait
/// in both testing and production contexts.
///
/// [module documentation]: crate::service
///
/// # Implementing
///
/// This trait is automatically adopted by any types that implement both
/// [`HTTPGet`] and [`HTTPPost`], so you can define a trait like this:
///
/// ```
/// use hypertyper::{Auth, HTTPError, HTTPGet, HTTPPost, HTTPResult, HTTPService, IntoUrl};
/// use reqwest::StatusCode;
/// use serde::Serialize;
/// use serde::de::DeserializeOwned;
/// use std::fmt::Debug;
///
/// #[derive(Debug)]
/// pub struct MyHTTPService;
///
/// impl HTTPGet for MyHTTPService {
///     async fn get<U>(&self, uri: U) -> HTTPResult<String>
///     where
///         U: IntoUrl + Send,
///     {
///         println!("Hello, GET! {:?}", uri.into_url());
///         Ok(String::from("Hello, GET!"))
///     }
/// }
///
/// impl HTTPPost for MyHTTPService {
///     async fn post<U, D, R>(&self, uri: U, auth: &Auth, _data: &D) -> HTTPResult<R>
///     where
///         U: IntoUrl + Send,
///         D: Serialize + Sync,
///         R: DeserializeOwned,
///     {
///         print!("Hello, POST! {:?} {:?}", uri.into_url(), auth);
///         Err(HTTPError::Http(StatusCode::INTERNAL_SERVER_ERROR))
///     }
/// }
///
/// pub fn hello_service(service: impl HTTPService + Debug) {
///     println!("Hello, service! {:?}", service);
/// }
/// ```
///
/// Note that `HTTPService` is automatically implemented. Pretty cool, huh?
pub trait HTTPService: HTTPGet + HTTPPost {}

impl<T: HTTPGet + HTTPPost> HTTPService for T {}
