
pub use url::Url;

use yew::html::{Component, Env};
use stdweb::Value;
use stdweb::web::{self, IEventTarget};

/// TODO:
/// A handle which helps to cancel the router. Uses removeEventListener
pub struct RouterTask<CTX: 'static, COMP: Component<CTX>> {
    _handle1: web::EventListenerHandle,
    handle2: web::EventListenerHandle,
    history: web::History,
    route_fn: &'static Fn(RouteInfo) -> COMP::Message,
    window: web::Window,
}

/// State of the current route.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// Url
    pub url: Url,
    /// History state
    pub state: Value,
}

impl RouteInfo {
    /// Initialize the route state using the current window.
    fn new(url: Url, state: Value) -> RouteInfo {
        RouteInfo {
            url: url,
            state: state,
        }
    }
}

fn current_url(window: &web::Window) -> Url {
    // TODO: better error messages around unwraps
    let location = expect!(window.location(), "could not get location");
    let href = expect!(location.href(), "could not get href");
    expect!(Url::parse(&href), "location.href did not parse")
}

impl<'a, CTX: 'a, COMP: Component<CTX>> RouterTask<CTX, COMP> {
    /// Start the Routing Task in the environment.
    ///
    /// Ownership of this Task should typically be put in the `Model`.
    ///
    /// Routing will stop if this Task is dropped.
    pub fn new(
        env: &mut Env<'a, CTX, COMP>,
        route_fn: &'static Fn(RouteInfo) -> COMP::Message,
    ) -> Self
    {
        let window = web::window();
        let callback = env.send_back(route_fn);

        let callback1 = callback.clone();
        let callback2 = callback;

        let cl_window = window.clone();
        let handle1 = window
            .add_event_listener(move |event: web::event::PopStateEvent| {
                callback1.emit(RouteInfo::new(current_url(&cl_window), event.state()));
            });

        let cl_window = window.clone();
        let handle2 = window
            .add_event_listener(move |event: web::event::ResourceLoadEvent| {
                callback2.emit(RouteInfo::new(current_url(&cl_window), Value::Null));
            });

        RouterTask {
            _handle1: handle1,
            handle2: handle2,
            route_fn: route_fn,
            history: window.history(),
            window: window,
        }
    }

    /// Retrieve the current url of the application.
    pub fn current_url(&self) -> Url {
        current_url(&self.window)
    }

    /// Set the state of the history, including the url.
    ///
    /// This will _not_ trigger the router to change. If a state change is required
    /// it is the user's job to propogate the `Message`.
    pub fn push_state(&self, state: Value, title: &str, url: Url) -> COMP::Message {
        self.history.push_state(state.clone(), title, Some(url.as_str()));
        let info = RouteInfo {
            url: url,
            state: state,
        };
        (*self.route_fn)(info)
    }

    /// Push a hash based on the current url.
    pub fn push_hash(&self, hash: Option<&str>) -> COMP::Message {
        let mut url = current_url(&self.window);
        url.set_fragment(hash);
        self.push_state(Value::Null, "", url)
    }
}
