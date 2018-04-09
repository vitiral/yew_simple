//! This crate contains the implementation of a service for
//! a url router.

#[macro_use] extern crate yew;
#[macro_use] extern crate stdweb;
extern crate url;

use yew::html::{Callback, Component};
use yew::services::Task;
use stdweb::Value;
use stdweb::web::{self, IEventTarget};

/// TODO:
/// A handle which helps to cancel the router. Uses removeEventListener
pub struct RouterService<CTX: 'static, COMP: Component<CTX>, F: Fn(RouteInfo) -> COMP::Msg> {
    callback: Callback<RouteInfo>,
    route_fn: Rc<F>,
    window: web::Window,
    history: web::History,
}

/// State of the current route.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// Window location
    pub url: url::Url,
    /// History state
    pub state: Value,
}

impl RouteInfo {
    /// Initialize the route state using the current window.
    fn new(url: Url, state: Value) -> RouteInfo {
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
            callback: None,
            handle1: None,
            handle2: None,
            route_fn: None,
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
        let callback2 = callback.clone();

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
        self.callback = Some(callback);
    }

    /// Set the state of the history, including the url.
    ///
    /// This will only trigger a state change in the frontend if `emit == false`
    pub fn push_state(&self, emit: bool, state: Value, title: &str, url: Option<&str>) {
        let route_fn = match self.route_fn {
            Some(r) => r,
            None => panic!("Attempted to set_state without initializing router"),
        };
        let url = match url {
            Some(url) => url.to_string(),
            None => self.window.location().unwrap().href().unwrap(),
        };
        self.history.push_state(state.clone(), title, Some(&url));

        if update {
            let info = RouteInfo {
                location: parse_location(&url),
                state: state,
            };
            self.callback.as_ref().unwrap().emit(info)
        }
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
