// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025 Michael Dippery <michael@monkey-robot.com>

//! Testing utilities for HTTP services.
//!
//! [`HttpService`] provides a test HTTP service that returns static
//! responses without making any actual HTTP requests over a network. It
//! is useful to test HTTP clients in unit tests.
//!
//! [`TestDataLoader`] is an easy way to load and deserialize data that
//! can be used when making HTTP POST or PUT calls.
//!
//! See each struct's documentation for examples of common usage.

use crate::{Auth, HttpGet, HttpPost, HttpResult};
use reqwest::IntoUrl;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;

#[cfg(doc)]
use crate::HttpService;

/// A service useful for unit tests that return responses containing
/// test data.
///
/// # Usage
///
/// `HTTPTestService` can be used in unit tests to mock an HTTP service.
/// Often times you will design your HTTP client to take an [`HttpService`].
/// In production, this will be a "real" data structure capable of making
/// calls out to an actual HTTP server, but in unit tests, you will likely
/// want a test version that returns static responses without making any
/// actual requests over a network.
///
/// `HTTPTestService` provides one such implementation of a mock HTTP
/// service. Here is a simple use of it:
///
/// ```
/// use hypertyper::service::HttpService;
/// use hypertyper::service::testing::HttpTestService;
///
/// pub struct APIClient<T: HttpService> {
///     service: T,
/// }
///
/// impl<T: HttpService> APIClient<T> {
///     fn with_service(service: T) -> Self {
///         Self { service }
///     }
/// }
///
/// // Often times, you may want to implement a public default() or new()
/// // method that calls with_service(), and make with_service() private
/// // pub(crate), but that is up to you.
/// /*
/// impl APIClient<MyHTTPService> {
///     pub fn new() -> Self {
///         let service = MyHTTPService::new();
///         Self::with_service(service)
///     }
/// }
/// */
///
/// // In your unit tests:
/// let client = APIClient::with_service(HTTPTestService::new("tests/data/output"));
/// ```
///
/// ## Configuration
///
/// `HTTPTestService` expects to find a file system structure that matches
/// the URIs you will call in GET and POST requests. For example, say you
/// create a service:
///
/// ```
/// # use hypertyper::service::testing::HttpTestService;
/// let service = HTTPTestService::new("tests/data/output");
/// ```
///
/// And then you make a GET request:
///
/// ```
/// # use hypertyper::HttpGet;
/// # use hypertyper::service::testing::HttpTestService;
/// # let service = HTTPTestService::new("tests/data/output");
/// let response = service.get("/users/foo/about");
/// ```
///
/// `HTTPTestService` would load data from `tests/data/users/foo/about.json`,
/// relative to where you ran `cargo test`.
///
/// You can also make POST requests the same way:
///
/// ```
/// # use hypertyper::{Auth, HttpPost};
/// # use hypertyper::service::testing::{HttpTestService, TestDataLoader};
/// # use serde::{Deserialize, Serialize};
/// #
/// # let service = HTTPTestService::new("tests/data/output");
/// #
/// # #[derive(Deserialize, Serialize)]
/// # struct User {
/// #     username: String,
/// # }
/// #
/// let loader = TestDataLoader::new("tests/data/input");
/// let auth = Auth::new("my-api-key");
/// let data: User = loader.load("user");
/// let response = service.post::<&str, User, User>("/users", &auth, &data);
/// ```
///
/// And `HTTPTestService` would deserialize the data in `tests/data/users.json`
/// and return the deserialized object in the response.
pub struct HttpTestService {
    root: String,
    ext: String,
}

impl HttpTestService {
    /// Creates a new test service that loads data from the `root` directory
    /// for its responses.
    pub fn new(root: impl Into<String>) -> Self {
        let root = root.into();
        let ext = String::from("json"); // TODO: Allow callers to specify
        Self { root, ext }
    }

    fn load_resource(&self, uri: impl IntoUrl + Send) -> String {
        let path = format!("{}{}.{}", self.root, uri.as_str(), self.ext);
        fs::read_to_string(path).expect("could not find test data")
    }
}

impl HttpGet for HttpTestService {
    /// Mocks an HTTP GET request by loading test data mapped to the given `uri`.
    ///
    /// # Panics
    ///
    /// If test data cannot be loaded.
    async fn get<U>(&self, uri: U) -> HttpResult<String>
    where
        U: IntoUrl + Send,
    {
        Ok(self.load_resource(uri).trim().to_string())
    }
}

impl HttpPost for HttpTestService {
    /// Mocks an HTTP POST request by loading test data mapped to the given `uri`.
    ///
    /// This method does nothing with the POST `data` itself, nor does it
    /// operate on `auth`; it just loads a response from the file system.
    ///
    /// # Panics
    ///
    /// If test data cannot be loaded.
    async fn post<U, D, R>(&self, uri: U, _auth: &Auth, _data: &D) -> HttpResult<R>
    where
        U: IntoUrl + Send,
        D: Serialize + Sync,
        R: DeserializeOwned,
    {
        let data = self.load_resource(uri);
        Ok(serde_json::from_str(&data)?)
    }
}

/// Loads data for mock test responses from your local file system.
///
/// # Usage
///
/// Say you have a resource stored at `tests/data/resource.json` in the
/// root of your repository (or wherever you typically run `cargo test` from).
/// You can easily load and deserialize that data using `TestDataLoader`:
///
/// ```
/// # use hypertyper::service::testing::TestDataLoader;
/// # use serde::Deserialize;
/// #
/// # #[derive(Deserialize)]
/// # struct Resource {
/// #     foo: String,
/// # }
/// #
/// let loader = TestDataLoader::new("tests/data/input");
/// let data: Resource = loader.load("resource");
/// ```
///
/// `TestDataLoader` is often used in conjunction with `HTTPService::post()`:
///
/// ```
/// # use hypertyper::{Auth, HttpPost};
/// # use hypertyper::service::testing::{HttpTestService, TestDataLoader};
/// # use serde::{Deserialize, Serialize};
/// #
/// # #[derive(Deserialize, Serialize)]
/// # struct Resource {
/// #     foo: String,
/// # }
/// #
/// let auth = Auth::new("my-api-key");
/// let loader = TestDataLoader::new("tests/data/input");
/// let data: Resource = loader.load("resource");
/// let service = HTTPTestService::new("tests/data/output");
/// let response = service.post::<&str, Resource, Resource>("/resources/1", &auth, &data);
/// ```
pub struct TestDataLoader {
    root: String,
    ext: String,
}

impl TestDataLoader {
    /// Create a new loader that loads test data from the `root` directory.
    pub fn new(root: impl Into<String>) -> Self {
        let root = root.into();
        let ext = String::from("json"); // TODO: Allow callers to specify
        Self { root, ext }
    }
}

impl TestDataLoader {
    /// Loads test data and serializes it into an object.
    ///
    /// # Panics
    ///
    /// If the test data cannot be loaded.
    pub fn load<T>(&self, resource: impl Into<String>) -> T
    where
        T: DeserializeOwned,
    {
        let resource = resource.into();
        let path = format!("{}/{resource}.{}", self.root, self.ext);
        let data = fs::read_to_string(path).expect("could not read test data");
        serde_json::from_str(&data).expect("could not deserialize test data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Auth, HttpError, HttpGet, HttpPost};
    use serde::{Deserialize, Serialize};
    use std::sync::LazyLock;

    static LOADER: LazyLock<TestDataLoader> =
        LazyLock::new(|| TestDataLoader::new("tests/data/input"));
    static SERVICE: LazyLock<HttpTestService> =
        LazyLock::new(|| HttpTestService::new("tests/data/output"));

    #[derive(Debug, Deserialize, Serialize)]
    struct User {
        username: String,
    }

    #[tokio::test]
    async fn get_loads_data() -> Result<(), HttpError> {
        let response = SERVICE.get("/users/foo/about").await?;
        assert_eq!(response, "{\"username\": \"foo\"}");
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_panics_if_data_does_not_exist() {
        let _ = SERVICE.get("/no-resource").await;
    }

    #[tokio::test]
    async fn post_loads_data() -> Result<(), HttpError> {
        let auth = Auth::new("my-api-key");
        let data: User = LOADER.load("user");
        let response: User = SERVICE.post("/users", &auth, &data).await?;
        assert_eq!(response.username, "foo");
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn post_panics_if_input_data_does_not_exist() {
        let auth = Auth::new("my-api-key");
        let data: User = LOADER.load("no-resource");
        let _: Result<User, _> = SERVICE.post("/users", &auth, &data).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn post_panics_if_output_data_does_not_exist() {
        let auth = Auth::new("my-api-key");
        let data: User = LOADER.load("user");
        let _: Result<User, _> = SERVICE.post("/admin", &auth, &data).await;
    }
}
