# Hypertyper

**Hypertyper** is a collection of useful types and idioms (and a few
implementations) for building HTTP/S clients. It provides convenient ways to
build and use HTTP/S clients. Configure an `HTTPClientFactory` once and use
it to produce as many `HTTPClient` instances as needed. Use `HTTPResult` to
provide a common way to return HTTP response bodies or errors, and wrap HTTP
errors in a common `HTTPError` enum to unify your HTTP response handling.

Under the hood, Hypertyper uses the excellent [reqwest] library to satisfy
all your HTTP needs.

Hypertyper was created to wrap the most common HTTP-related code into a
common interface usable across libraries and applications. It is a
rapidly-evolving project that will grow to encapsulate the most common
HTTP types, idioms, and operations, allowing you to focus on the specific
needs of your applications.

[reqwest]: https://crates.io/crates/reqwest
