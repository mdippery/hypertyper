// SPDX-License-Identifier: MIT
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
//! use hypertyper::{HTTPClient, HTTPClientFactory};
//! use hypertyper::auth::Auth;
//! use hypertyper::service::HTTPService;
//!
//! pub struct RealService {
//!   auth: Auth,
//!   client: HTTPClient,
//! }
//!
//! impl RealService {
//!   pub fn new(auth: Auth, factory: HTTPClientFactory) -> Self {
//!     let client = factory.create();
//!     Self { auth, client }
//!   }
//! }
//!
//! impl HTTPService for RealService {}
//!
//! #[derive(Default)]
//! pub struct TestService;
//!
//! impl HTTPService for TestService {}
//!
//! pub struct APIClient<S: HTTPService> {
//!   service: S,
//! }
//!
//! impl<S: HTTPService> APIClient<S> {
//!   // Note: Not pub! This will only be available in tests within the module.
//!   // Use pub(crate) fn if with_service() should be available in test utils
//!   // modules, too.
//!   fn with_service(service: S) -> Self {
//!     Self { service }
//!   }
//! }
//!
//! impl APIClient<RealService> {
//!   pub fn new() -> Self {
//!     let auth = Auth::from_env("MY_COOL_API_KEY");
//!     let factory = HTTPClientFactory::with_user_agent("my cool user agent");
//!     let service = RealService::new(auth, factory);
//!     Self::with_service(service)
//!   }
//! }
//!
//! let real_client = APIClient::new();
//!
//! // APIClient::with_service() is only available within the module,
//! // which simplifies the public API while allowing easy testing.
//! let test_client = APIClient::with_service(TestService::default());
//! ```
//!
//! Together, an HTTP service trait and its various concrete implementations
//! provide a uniform way of communicating over HTTP, whether code is
//! under test or live in production.
