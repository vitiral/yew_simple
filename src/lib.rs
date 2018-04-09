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
pub struct RouterService<CTX: 'static, COMP: Component<CTX>> {
    handle1: Option<web::EventListenerHandle>,
    // handle2: Option<web::EventListenerHandle>,
    handle2: Option<Value>,
    route_fn: Option<&'static Fn(RouteInfo) -> COMP::Msg>,
    window: web::Window,
    history: web::History,
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

impl<CTX: 'static, COMP: Component<CTX>> RouterService<CTX, COMP> {
    /// Creates a new service instance connected to `App` by provided `sender`.
    pub fn new() -> Self {
        let window = web::window();
        RouterService {
            handle1: None,
            handle2: None,
            route_fn: None,
            history: window.history(),
            window: window,
        }
    }

    pub fn create(
        env: &mut Env<'static, CTX, COMP>,
        route_fn: &'static Fn(RouteInfo) -> COMP::Msg,
    ) -> Self
    {
        let window = web::window();
        let callback = env.send_back(route_fn);

        RouterService {
            handle1: None,
            handle2: None,
            route_fn: Some(route_fn),
            history: window.history(),
            window: window,
        }
    }

    /// Sets the router, which will continuously use the callback to route all relevant messages,
    /// as well as when `set_state` is called.
    ///
    /// Typically this is called fron within the `impl Component::create for Model`.
    ///
    /// `env_callback` and `route_fn` must use the same function.
    pub fn initialize(
        &mut self,
        callback: Callback<RouteInfo>,
        route_fn: &'static Fn(RouteInfo) -> COMP::Msg,
    ) {
        let callback1 = callback.clone();
        let callback2 = callback;

        self.handle1 = Some(self.window
            .add_event_listener(move |event: web::event::PopStateEvent| {
                callback1.emit(RouteInfo::new(event.state()));
            }));

        // TODO: koute/stdweb/issues/171
        // self.handle2 = Some(self.window
        //     .add_event_listener(move |_event: web::event::ResourceLoadEvent| {
        //         callback2.emit(RouteInfo::new(Value::Null));
        //     }));

        let rs_handle = move || {
            callback2.emit(RouteInfo::new(Value::Null));
        };

        self.handle2 = Some(js!{
            // FIXME: for some _bizare_ reason defining `action` is necessary here...
            function action() { @{rs_handle()} };
            window.addEventListener("load", action);
            return action;

            // What I want to do is this:
            // https://github.com/koute/stdweb/issues/171#issuecomment-379512282
            // var handle = @{rs_handle};
            // window.addEventListener("load", handle);
            // return handle;
        });

        self.route_fn = Some(route_fn);
    }

    /// Set the state of the history, including the url.
    ///
    /// This will _not_ trigger the router to change. If a state change is required
    /// it is the user's job to propogate the `Msg`.
    pub fn push_state(&self, state: Value, title: &str, url: Option<&str>) -> COMP::Msg {
        let route_fn = match self.route_fn {
            Some(r) => r,
            None => panic!("Attempted to set_state without initializing router"),
        };
        let url = match url {
            Some(url) => url.to_string(),
            None => self.window.location().unwrap().href().unwrap(),
        };
        self.history.push_state(state.clone(), title, Some(&url));
        let info = RouteInfo {
            location: parse_location(&url),
            state: state,
        };
        route_fn(info)
    }
}

impl<CTX, COMP: Component<CTX>> Task for RouterService<CTX, COMP> {
    fn is_active(&self) -> bool {
        self.handle1.is_some()
    }

    fn cancel(&mut self) {
        self.handle1
            .take()
            .expect("tried to cancel interval twice")
            .remove();

        // TODO: koute/stdweb/issues/171
        // self.handle2
        //     .take()
        //     .expect("tried to cancel interval twice")
        //     .remove();
        js! { @(no_return)
            let handle = @{self.handle2.take().unwrap()};
            window.removeEventListener("load", handle);
        };
        self.route_fn.take();
    }
}

impl<CTX, COMP: Component<CTX>> Drop for RouterService<CTX, COMP> {
    fn drop(&mut self) {
        if self.is_active() {
            self.cancel();
        }
    }
}
