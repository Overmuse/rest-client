//! rest-client is a library for building strongly typed REST clients, with built-in capabilites
//! for authentication, various request and response types and pagination.
//!
//! Inspired heavily by [ring-api](https://github.com/H2CO3/ring_api)
mod client;
mod error;
mod pagination;
mod request;

pub use client::Client;
pub use error::Error;
pub use pagination::*;
pub use request::*;
