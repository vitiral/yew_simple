//! Types from the `window`
use stdweb::web;

/// The `window.location` object with all fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    /// href
    pub href: String,
    // TODO: these are being added to stdweb
    // pub origin: String,
    // pub protocol: String,
    // pub host: String,
    // pub hostname: String,
    // pub port: String,
    // pub pathname: String,
    // pub search: String,
    /// hash
    pub hash: String,
}

pub(crate) fn parse_location(s: &str) -> Location {
    let location = js!{
        return new URL(@{s.to_string()});
    };
    Location {
        href: js!{return @{&location}.href()}.into_string().unwrap(),
        hash: js!{return @{&location}.hash()}.into_string().unwrap(),
    }
}

/// Get `window.location`
pub(crate) fn get_location(window: &web::Window) -> Location {
    // TODO: log security error for security error
    // TODO: log error when location DNE

    // TODO: want to do this in one big call, not sure how to convert
    // js!(
    //     var location = window.location;
    //     return {
    //         href: location.href,
    //         origin: location.origin,
    //         protocol: location.protocol,
    //         host: location.host,
    //         hostname: location.hostname,
    //         port: location.port,
    //         pathname: location.pathname,
    //         search: location.search,
    //         hash: location.hash,
    //     };
    // )
    let location = window.location().unwrap();
    Location {
        href: location.href().unwrap(),
        // TODO: these are being added to stdweb
        // origin: location.origin().unwrap(),
        // protocol: location.protocol().unwrap(),
        // host: location.host().unwrap(),
        // hostname: location.hostname().unwrap(),
        // port: location.port().unwrap(),
        // pathname: location.pathname().unwrap(),
        // search: location.search().unwrap(),
        hash: location.hash().unwrap(),
    }
}
