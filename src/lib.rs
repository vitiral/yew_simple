//! This module contains simple services and tasks for the [`yew`] framework.
//!
//! [`yew`]: https://github.com/DenisKolodin/yew

extern crate yew;
#[macro_use] extern crate stdweb;
pub extern crate url;

mod router;

pub use router::{RouterTask, RouteInfo};
