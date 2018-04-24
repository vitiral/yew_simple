//! This module contains simple services and tasks for the [`yew`] framework.
//!
//! [`yew`]: https://github.com/DenisKolodin/yew
#![recursion_limit="256"]

extern crate yew;
#[macro_use] extern crate stdweb;
pub extern crate url;
pub extern crate http;

mod router;
mod fetch;

pub use router::{RouterTask, RouteInfo};
