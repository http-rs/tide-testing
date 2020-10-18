#![forbid(unsafe_code, future_incompatible)]
#![deny(
    missing_debug_implementations,
    nonstandard_style,
    missing_copy_implementations,
    unused_qualifications
)]
//! # Tide Testing Extension Trait
//!
//! This trait provides an ergonomic extension for testing tide applications.
//!
//! ## Usage:
//!
//! `$ cargo add -D tide-testing`
//!
//! ## Examples
//!
//! ```rust
//! # fn main() -> tide::Result<()> { async_std::task::block_on(async {
//! let mut app = tide::new();
//! app.at("/").get(|_| async { Ok("hello!") });
//!
//! use tide_testing::TideTestingExt;
//! assert_eq!(app.get("/").recv_string().await?, "hello!");
//! assert_eq!(
//!     app.post("/missing").await?.status(),
//!     tide::http::StatusCode::NotFound
//! );
//! # Ok(()) }) }
//! ```
//!
//! Note that the http methods return [`surf::RequestBuilder`]s, allowing tests to build up complex requests fluently:
//!
//! ```rust
//! # fn main() -> tide::Result<()> {  async_std::task::block_on(async {
//! use tide::prelude::*;
//! let mut app = tide::new();
//! app.at("/complex_example")
//!     .put(|mut request: tide::Request<()>| async move {
//!         Ok(json!({
//!             "content_type": request.content_type().map(|c| c.to_string()),
//!             "body": request.body_string().await?,
//!             "header": request.header("custom").map(|h| h.as_str())
//!         }))
//!     });
//!
//! use tide_testing::TideTestingExt;
//!
//! let response_body: serde_json::value::Value = app
//!     .put("/complex_example")
//!     .body(tide::Body::from_string("hello".into()))
//!     .content_type("application/custom")
//!     .header("custom", "header-value")
//!     .recv_json()
//!     .await?;
//!
//! assert_eq!(
//!     response_body,
//!     json!({
//!         "content_type": Some("application/custom"),
//!         "body": "hello",
//!         "header": Some("header-value")
//!     })
//! );
//! # Ok(()) }) }
//! ```

pub use surf;
use surf::http::Url;
use surf::Client;
use tide::Server;

macro_rules! method {
    ($method:ident) => {
        fn $method(&self, url: impl AsRef<str>) -> surf::RequestBuilder {
            self.client().$method(url.as_ref())
        }
    };
}

macro_rules! methods {
    ($($method:ident),* $(,)?) => {$(method!($method);)+}
}

pub trait TideTestingExt {
    fn client(&self) -> Client;
    methods!(get, put, post, delete, head, connect, options, trace, patch);
}

impl<State: Unpin + Clone + Send + Sync + 'static> TideTestingExt for Server<State> {
    fn client(&self) -> Client {
        let mut client = Client::with_http_client(self.clone());
        client.set_base_url(Url::parse("http://example.com/").unwrap());
        client
    }
}
