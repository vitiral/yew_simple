//! This module contains simple services and tasks for the [`yew`] framework.
//!
//! [`yew`]: https://github.com/DenisKolodin/yew
#![recursion_limit="256"]

#[macro_use] extern crate expect_macro;
pub extern crate http;
#[macro_use] extern crate stdweb;
pub extern crate url;
extern crate yew;

mod router;
mod fetch;

pub use router::{RouterTask, RouteInfo};
pub use fetch::FetchTask;
