//!
//! A micro Web framework based on Hyper, Futures and Tokio.
//!
//! ## WARNING
//! This project is not production ready.
//!

#[doc(hidden)]
pub extern crate futures;
#[doc(hidden)]
pub extern crate hyper;
extern crate regex;
extern crate tokio_core;
#[doc(hidden)]
pub extern crate typemap;

pub mod context;
pub mod middleware;
pub mod response;
pub mod router;
pub mod susanoo;

pub mod contrib {
    pub use futures;
    pub use hyper;
    pub use typemap;
}

#[doc(inline)]
pub use context::Context;
#[doc(inline)]
pub use middleware::{Middleware, MiddlewareStack};
#[doc(inline)]
pub use response::{Response, Failure, AsyncResult};
#[doc(inline)]
pub use susanoo::Susanoo;
#[doc(inline)]
pub use router::{Router, Captures};
