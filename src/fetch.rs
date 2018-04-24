//! Service to send HTTP-request to a server.
//!
//! Based on code from <https://github.com/DenisKolodin/yew>'s `services/fetch.rs`

use std::collections::HashMap;

use stdweb::Value;
use stdweb::unstable::{TryFrom, TryInto};

use yew::services::Task;
use yew::format::{Restorable, Storable};
use yew::callback::Callback;

use http::{HeaderMap, Method, StatusCode, Uri};
use http;

/// A handle to control sent requests. Can be canceled with a `Task::cancel` call.
pub struct FetchTask(Option<Value>);

/// A service to fetch resources.
#[derive(Default)]
pub struct FetchService {}

impl FetchTask {

    /// Sends a request to a remote server given a Request object and a callback
    /// fuction to convert a Response object into a loop's message.
    ///
    /// You may use a Request builder to build your request declaratively as on the
    /// following examples:
    ///
    ///
    /// # Examples
    ///
    /// ```rust
    ///    let post_request = Request::post("https://my.api/v1/resource")
    ///            .header("Content-Type", "application/json")
    ///            .body("Some data".to_string())
    ///            .expect("Failed to build request.");
    /// ```
    ///
    /// The callback function can build a loop message by passing or analizing the
    /// response body and metadata.
    ///
    /// ```rust
    ///     FetchTask::new(
    ///         post_request,
    ///         |response| {
    ///             if response.status().is_success() {
    ///                 Msg::RecvData(response.into_body())
    ///             } else {
    ///                 Msg::Error
    ///             }
    ///         }
    ///     )
    /// ```
    pub fn new(
        request: http::Request<String>,
        yew_callback: Callback<http::Response<String>>,
    ) -> FetchTask
    {
       // Consume request as parts and body.
       let (parts, body): (_, String) = request.into_parts();

       // Map headers into a Js serializable HashMap.
       let header_map: HashMap<&str, &str> = parts
           .headers
           .iter()
           .map(|(k, v)| {
               let v = expect!(
                   v.to_str(),
                   "Unparsable request header {}: {:?}", k.as_str(), v
               );
               (
                   k.as_str(),
                   v,
               )
           })
           .collect();
       // Formats URI.
       let uri = format!("{}", parts.uri);

       // Prepare the response callback.
       // Notice that the callback signature must match the call from the javascript
       // side. There is no static check at this point.
       let js_callback = move |success: bool, response: Value, recv_body: String| -> () {
           let mut response_builder = http::Response::builder();

           // Deserialize response status.
           let status = u16::try_from(js!{
               return @{&response}.status;
           });

           if let Ok(code) = status {
               response_builder.status(code);
           }

           // Deserialize response headers.
           let headers: HashMap<String, String> = HashMap::try_from(js!{
               var map = {};
               @{&response}.headers.forEach(function(value, key) {
                   map[key] = value;
               });
               return map;
           }).unwrap_or_default();

           for (key, values) in &headers {
               response_builder.header(key.as_str(), values.as_str());
           }

           // Deserialize and wrap response body into a Restorable object.
           let response = response_builder.body(recv_body).unwrap();
           yew_callback.emit(response);
       };

       let handle = js! {
           var data = {
               // should this be to_string()?
               method: @{parts.method.as_str()},
               body: @{body},
               headers: @{header_map},
           };
           var request = new Request(@{uri}, data);
           var callback = @{js_callback};
           var handle = {
               active: true,
               callback: callback,
           };
           fetch(request).then(function(response) {
               response.text().then(function(data) {
                   if (handle.active == true) {
                       handle.active = false;
                       callback(true, response, data);
                       callback.drop();
                   }
               }).catch(function(err) {
                   if (handle.active == true) {
                       handle.active = false;
                       callback(false, response, data);
                       callback.drop();
                   }
               });
           });
           return handle;
       };
       FetchTask(Some(handle))
    }
}

impl Task for FetchTask {
    fn is_active(&self) -> bool {
        if let Some(ref task) = self.0 {
            let result = js! {
                var the_task = @{task};
                return the_task.active;
            };
            result.try_into().unwrap_or(false)
        } else {
            false
        }
    }
    fn cancel(&mut self) {
        // Fetch API doesn't support request cancelling
        // and we should use this workaround with a flag.
        // In fact, request not canceled, but callback won't be called.
        let handle = self.0
            .take()
            .expect("tried to cancel request fetching twice");
        js! {  @(no_return)
            var handle = @{handle};
            handle.active = false;
            handle.callback.drop();
        }
    }
}

impl Drop for FetchTask {
    fn drop(&mut self) {
        if self.is_active() {
            self.cancel();
        }
    }
}
