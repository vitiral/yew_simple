//! This module contains the implementation of a service for
//! a url router.

#[macro_use] extern crate yew;
#[macro_use] extern crate stdweb;


mod window;

use std::rc::Rc;

use yew::html::{Callback, Component, Env};
use yew::services::Task;
use stdweb::Value;
use stdweb::web::{self, IEventTarget};

use window::{get_location, parse_location};
pub use window::Location;


/// TODO:
/// A handle which helps to cancel the router. Uses removeEventListener
pub struct RouterTask<CTX: 'static, COMP: Component<CTX>> {
    handle1: web::EventListenerHandle,
    handle2: Value,
    history: web::History,
    route_fn: &'static Fn(RouteInfo) -> COMP::Msg,
    window: web::Window,
}

/// State of the current route.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// Window location
    pub location: Location,
    /// History state
    pub state: Value,
}

impl RouteInfo {
    /// Initialize the route state using the current window.
    fn new(state: Value) -> RouteInfo {
        let window = web::window();
        let location = get_location(&window);
        RouteInfo {
            location: location,
            state: state,
        }
    }
}

impl<'a, CTX: 'a, COMP: Component<CTX>> RouterTask<CTX, COMP> {
    /// Start the Routing Task in the environment.
    ///
    /// Ownership of this Task should typically be put in the `Model`.
    ///
    /// Routing will stop if this Task is dropped.
    pub fn new(
        env: &mut Env<'a, CTX, COMP>,
        route_fn: &'static Fn(RouteInfo) -> COMP::Msg,
    ) -> Self
    {
        let window = web::window();
        let callback = env.send_back(route_fn);

        let callback1 = callback.clone();
        let callback2 = callback;

        let handle1 = window
            .add_event_listener(move |event: web::event::PopStateEvent| {
                callback1.emit(RouteInfo::new(event.state()));
            });

        // TODO: koute/stdweb/issues/171
        // self.handle2 = Some(self.window
        //     .add_event_listener(move |_event: web::event::ResourceLoadEvent| {
        //         callback2.emit(RouteInfo::new(Value::Null));
        //     }));

        let rs_handle = move || {
            callback2.emit(RouteInfo::new(Value::Null));
        };

        let handle2 = js!{
            var callback = @{rs_handle};
            function listener() {
                callback();
            }
            window.addEventListener("load", listener);
            return {
                callback: callback,
                listener: listener
            };
        };

        RouterTask {
            handle1: handle1,
            handle2: handle2,
            route_fn: route_fn,
            history: window.history(),
            window: window,
        }
    }

    /// Set the state of the history, including the url.
    ///
    /// This will _not_ trigger the router to change. If a state change is required
    /// it is the user's job to propogate the `Msg`.
    pub fn push_state(&self, state: Value, title: &str, url: Option<&str>) -> COMP::Msg {
        let url = match url {
            Some(url) => url.to_string(),
            None => self.window.location().unwrap().href().unwrap(),
        };
        self.history.push_state(state.clone(), title, Some(&url));
        let info = RouteInfo {
            location: parse_location(&url),
            state: state,
        };
        let route_fn = self.route_fn;
        route_fn(info)
    }
}

impl<CTX, COMP: Component<CTX>> Drop for RouterTask<CTX, COMP> {
    fn drop(&mut self) {
        js! { @(no_return)
            var handle = @{&self.handle2};
            window.removeEventListener("load", handle.listener);
            handle.callback.drop();
        }
    }
}
