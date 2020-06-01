// Most of code in this module is based on Yew's fetch service
pub use http::Request;
use wasm_bindgen::UnwrapThrowExt;

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error("Invalid request header")]
    InvalidRequestHeader,
    #[error("Build headers failed")]
    BuildHeaderFailed,
    #[error("Build request failed")]
    BuildRequestFailed,
    #[error("Fetch failed")]
    FetchFailed,
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    #[error("Response status: {}", .0)]
    ResponseWithError(http::StatusCode),
    #[error("Invalid response")]
    InvalidResponse,
    #[error("Empty response")]
    EmptyResponse,
    #[error("Convert to json error: {0}")]
    JsonError(serde_json::error::Error),
    #[error("Parse from json error: {0}")]
    ParseJsonError(serde_json::error::Error),
    /// Error return by http crate
    #[error("Http error: {0}")]
    HttpError(#[from] http::Error),
}

#[derive(Debug)]
pub enum Referrer {
    /// `<same-origin URL>` value of referrer.
    SameOriginUrl(String),
    /// `about:client` value of referrer.
    AboutClient,
    /// `<empty string>` value of referrer.
    Empty,
}

/// Init options for `fetch()` function call.
/// https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/fetch
#[derive(Default, Debug)]
pub struct FetchOptions {
    /// Cache of a fetch request.
    pub cache: Option<web_sys::RequestCache>,
    /// Credentials of a fetch request.
    pub credentials: Option<web_sys::RequestCredentials>,
    /// Redirect behaviour of a fetch request.
    pub redirect: Option<web_sys::RequestRedirect>,
    /// Request mode of a fetch request.
    pub mode: Option<web_sys::RequestMode>,
    /// Referrer of a fetch request.
    pub referrer: Option<Referrer>,
    /// Referrer policy of a fetch request.
    pub referrer_policy: Option<web_sys::ReferrerPolicy>,
    /// Integrity of a fetch request.
    pub integrity: Option<String>,
}

impl Into<web_sys::RequestInit> for FetchOptions {
    fn into(self) -> web_sys::RequestInit {
        let mut init = web_sys::RequestInit::new();

        if let Some(cache) = self.cache {
            init.cache(cache);
        }

        if let Some(credentials) = self.credentials {
            init.credentials(credentials);
        }

        if let Some(redirect) = self.redirect {
            init.redirect(redirect);
        }

        if let Some(mode) = self.mode {
            init.mode(mode);
        }

        if let Some(referrer) = self.referrer {
            match referrer {
                Referrer::SameOriginUrl(referrer) => init.referrer(&referrer),
                Referrer::AboutClient => init.referrer("about:client"),
                Referrer::Empty => init.referrer(""),
            };
        }

        if let Some(referrer_policy) = self.referrer_policy {
            init.referrer_policy(referrer_policy);
        }

        if let Some(integrity) = self.integrity {
            init.integrity(&integrity);
        }

        init
    }
}

fn build_request(
    parts: http::request::Parts,
    body: Option<&wasm_bindgen::JsValue>,
) -> Result<web_sys::Request, FetchError> {
    use std::iter::FromIterator;

    // Map headers into a Js `Header` type.
    let header_list = parts
        .headers
        .iter()
        .map(|(k, v)| {
            Ok(js_sys::Array::from_iter(&[
                wasm_bindgen::JsValue::from_str(k.as_str()),
                wasm_bindgen::JsValue::from_str(
                    v.to_str().map_err(|_| FetchError::InvalidRequestHeader)?,
                ),
            ]))
        })
        .collect::<Result<js_sys::Array, FetchError>>()?;

    let header_map = web_sys::Headers::new_with_str_sequence_sequence(&header_list)
        .map_err(|_| FetchError::BuildHeaderFailed)?;

    // Formats URI.
    let uri = parts.uri.to_string();
    let method = parts.method.as_str();
    let mut init = web_sys::RequestInit::new();
    init.method(method).body(body).headers(&header_map);
    web_sys::Request::new_with_str_and_init(&uri, &init).map_err(|_| FetchError::BuildRequestFailed)
}

async fn fetch_async<F1, F2, R>(promise: js_sys::Promise, ok_handler: F1, error_handler: F2)
where
    R: 'static + serde::de::DeserializeOwned,
    F1: FnOnce(R),
    F2: FnOnce(FetchError),
{
    match get_string(promise)
        .await
        .and_then(|data| serde_json::from_str::<R>(&data).map_err(FetchError::ParseJsonError))
    {
        Ok(data) => ok_handler(data),
        Err(e) => error_handler(e),
    }
}

async fn get_string(promise: js_sys::Promise) -> Result<String, FetchError> {
    let response = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| FetchError::FetchFailed)?;

    let response = web_sys::Response::from(response);
    let status = http::StatusCode::from_u16(response.status())?;
    if !status.is_success() {
        return Err(FetchError::ResponseWithError(status));
    }

    let promise = response.text().map_err(|_| FetchError::InvalidResponse)?;

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|js_text| js_text.as_string())
        .map_err(|_| FetchError::InvalidResponse)?
        .ok_or_else(|| FetchError::EmptyResponse)
}

pub enum OkHandler<C: crate::component::Component, Cl, R> {
    OnlyArg(fn(&mut C, R) -> Cl),
    ChildCompsAndArg(fn(&mut C, &mut C::Components, R) -> Cl),
}

pub struct FetchArgs {
    request_builder: http::request::Builder,
    options: Option<FetchOptions>,
    body: Option<Result<String, FetchError>>,
}

pub trait IntoFetchArgs {
    fn into_fetch_args(self) -> FetchArgs;
}

impl IntoFetchArgs for http::request::Builder {
    fn into_fetch_args(self) -> FetchArgs {
        FetchArgs::new(self)
    }
}

impl FetchArgs {
    pub fn new(request_builder: http::request::Builder) -> Self {
        Self {
            request_builder,
            options: None,
            body: None,
        }
    }

    pub fn options(mut self, options: FetchOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn json_body<T: serde::Serialize>(mut self, data: &T) -> Self {
        if let Some(headers) = self.request_builder.headers_mut() {
            headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/json"),
            );
            self.body = Some(serde_json::to_string(&data).map_err(FetchError::JsonError));
        }
        self
    }

    pub fn json_response<C, T, Cl>(
        self,
        ok_handler: fn(&mut C, T) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, T, Cl>>
    where
        C: crate::component::Component,
        T: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        let parts = self
            .request_builder
            .body(())
            .map(|r| r.into_parts().0)
            .map_err(From::from);
        Box::new(FetchCmd(Some(FetchCmdArgs {
            parts,
            options: self.options.unwrap_or_else(Default::default),
            body: self.body,
            ok_handler: OkHandler::OnlyArg(ok_handler),
            error_handler,
        })))
    }

    pub fn json_response_then_provide_child_comps<C, T, Cl>(
        self,
        ok_handler: fn(&mut C, &mut C::Components, T) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, T, Cl>>
    where
        C: crate::component::Component,
        T: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        let parts = self
            .request_builder
            .body(())
            .map(|r| r.into_parts().0)
            .map_err(From::from);
        Box::new(FetchCmd(Some(FetchCmdArgs {
            parts,
            options: self.options.unwrap_or_else(Default::default),
            body: self.body,
            ok_handler: OkHandler::ChildCompsAndArg(ok_handler),
            error_handler,
        })))
    }
}

pub struct FetchCmd<C: crate::component::Component, T, Cl>(Option<FetchCmdArgs<C, T, Cl>>);

struct FetchCmdArgs<C: crate::component::Component, T, Cl> {
    parts: Result<http::request::Parts, FetchError>,
    options: FetchOptions,
    body: Option<Result<String, FetchError>>,
    ok_handler: OkHandler<C, Cl, T>,
    error_handler: fn(&mut C, FetchError),
}

impl<C, T, Cl> crate::component::Command<C> for FetchCmd<C, T, Cl>
where
    C: 'static + crate::component::Component,
    T: 'static + serde::de::DeserializeOwned,
    Cl: 'static + Into<crate::component::Checklist<C>>,
{
    fn execute(&mut self, comp: &crate::component::Comp<C>, state: &mut C) {
        let FetchCmdArgs {
            parts,
            body,
            options,
            ok_handler,
            error_handler,
        } = self
            .0
            .take()
            .expect_throw("Why FetchCmd is executed twice?");

        // Transform http::Request into web_sys::Request.
        let body = match body {
            Some(Ok(body)) => Some(wasm_bindgen::JsValue::from(body)),
            Some(Err(e)) => {
                error_handler(state, e);
                return;
            }
            None => None,
        };

        let parts = match parts {
            Ok(parts) => parts,
            Err(e) => {
                error_handler(state, e);
                return;
            }
        };

        let ws_request = match build_request(parts, body.as_ref()) {
            Ok(request) => request,
            Err(e) => {
                // The component instance is currently being borrowed,
                // we must send the error via the `state`, not the `comp`.
                error_handler(state, e);
                return;
            }
        };

        // Transform FetchOptions into RequestInit.
        //
        // Not care about aborting yet
        // let abort_controller = AbortController::new().ok();
        let init = options.into(); //.map_or_else(web_sys::RequestInit::new, Into::into);
                                   // if let Some(abort_controller) = &abort_controller {
                                   //     init.signal(Some(&abort_controller.signal()));
                                   // }

        // Start fetch
        let promise = crate::utils::window().fetch_with_request_and_init(&ws_request, &init);

        let error_handler = comp.callback_arg(error_handler);
        match ok_handler {
            OkHandler::OnlyArg(f) => {
                let ok_handler = comp.callback_arg(f);
                let f = fetch_async(promise, ok_handler, error_handler);
                wasm_bindgen_futures::spawn_local(f);
            }
            OkHandler::ChildCompsAndArg(f) => {
                let ok_handler = comp.callback_child_comps_arg(f);
                let f = fetch_async(promise, ok_handler, error_handler);
                wasm_bindgen_futures::spawn_local(f);
            }
        }
    }
}

// impl<C, T, Cl> From<Box<FetchCmd<C, T, Cl>>> for crate::component::Checklist<C>
// where
//     C: 'static + crate::component::Component,
//     T: 'static + serde::de::DeserializeOwned,
//     Cl: 'static + Into<crate::component::Checklist<C>>,
// {
//     fn from(cmd: Box<FetchCmd<C, T, Cl>>) -> Self {
//         let mut checklist = crate::component::Checklist::run_fn_render();
//         checklist.fetch(cmd);
//         checklist
//     }
// }
