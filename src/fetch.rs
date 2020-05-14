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
    #[error("Invalid response")]
    InvalidResponse,
    #[error("Empty response")]
    EmptyResponse,
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::error::Error),
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

pub struct FetchCommand<C, F, R> {
    args: Option<FetchCommandArgs<C, F>>,
    _phantom_data: std::marker::PhantomData<R>,
}

struct FetchCommandArgs<C, F> {
    request: http::Request<Option<String>>,
    options: FetchOptions,
    ok_handler: F,
    error_handler: fn(&mut C, FetchError),
}

impl<C, F, R> FetchCommand<C, F, R>
where
    C: crate::component::Component,
{
    pub(crate) fn new(
        request: http::Request<Option<String>>,
        options: Option<FetchOptions>,
        ok_handler: F,
        error_handler: fn(&mut C, FetchError),
    ) -> Self {
        Self {
            args: Some(FetchCommandArgs {
                request,
                options: options.unwrap_or_else(Default::default),
                ok_handler,
                error_handler,
            }),
            _phantom_data: std::marker::PhantomData,
        }
    }
}

impl<C, F, R, Cl> crate::component::Command<C> for FetchCommand<C, F, R>
where
    C: 'static + crate::component::Component,
    R: 'static + serde::de::DeserializeOwned,
    F: 'static + Fn(&mut C, R) -> Cl,
    Cl: 'static + Into<crate::component::Checklist<C>>,
{
    fn execute(&mut self, comp: &crate::component::Comp<C>, state: &mut C) {
        let FetchCommandArgs {
            request,
            options,
            ok_handler,
            error_handler,
        } = self
            .args
            .take()
            .expect_throw("Why FetchCommand is executed twice?");

        // Transform http::Request into web_sys::Request.
        let (parts, body) = request.into_parts();
        let body = body.and_then(|body| match serde_json::to_string(&body) {
            Ok(body) => Some(wasm_bindgen::JsValue::from(body)),
            Err(e) => {
                // The component instance is currently being borrowed,
                // we must send the error via the `state`, not the `comp`.
                error_handler(state, FetchError::from(e));
                None
            }
        });

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

        // Spawn future to resolve fetch
        // let active = Rc::new(RefCell::new(true));

        let ok_handler = comp.callback_arg(ok_handler);
        let error_handler = comp.callback_arg(error_handler);

        let f = fetch_async(promise, ok_handler, error_handler);
        wasm_bindgen_futures::spawn_local(f);
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
    web_sys::Request::new_with_str_and_init(&uri, &init).map_err(|e| {
        log::error!("{:?}", e);
        FetchError::BuildRequestFailed
    })
}

async fn fetch_async<F1, F2, R>(promise: js_sys::Promise, ok_handler: F1, error_handler: F2)
where
    R: 'static + serde::de::DeserializeOwned,
    F1: FnOnce(R),
    F2: FnOnce(FetchError),
{
    match get_string(promise)
        .await
        .and_then(|data| serde_json::from_str::<R>(&data).map_err(From::from))
    {
        Ok(data) => ok_handler(data),
        Err(e) => error_handler(e),
    }
}

async fn get_string(promise: js_sys::Promise) -> Result<String, FetchError> {
    let response = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| FetchError::FetchFailed)?;

    let promise = web_sys::Response::from(response)
        .text()
        .map_err(|_| FetchError::InvalidResponse)?;

    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|js_text| js_text.as_string())
        .map_err(|_| FetchError::InvalidResponse)?
        .ok_or_else(|| FetchError::EmptyResponse)
}
