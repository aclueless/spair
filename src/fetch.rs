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
    #[error("Response is not a string")]
    NotAString,
    #[error("Convert to json error: {0}")]
    EncodeJsonError(serde_json::error::Error),
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

pub trait IntoFetchArgs {
    #[deprecated(
        since = "0.0.5",
        note = "Replaced by .text_mode().body() or .text_mode().response()"
    )]
    fn into_fetch_args(self) -> FetchArgs;
}

impl IntoFetchArgs for http::request::Builder {
    fn into_fetch_args(self) -> FetchArgs {
        FetchArgs {
            request_builder: self,
            options: None,
            body: None,
        }
    }
}

pub struct FetchArgs {
    request_builder: http::request::Builder,
    options: Option<FetchOptions>,
    body: Option<Result<wasm_bindgen::JsValue, FetchError>>,
}

impl From<http::request::Builder> for FetchArgs {
    fn from(request_builder: http::request::Builder) -> Self {
        Self {
            request_builder,
            options: None,
            body: None,
        }
    }
}

pub trait FetchOptionsSetter {
    fn options(self, options: FetchOptions) -> FetchArgs;
}

impl FetchOptionsSetter for http::request::Builder {
    fn options(self, options: FetchOptions) -> FetchArgs {
        FetchArgs {
            request_builder: self,
            options: Some(options),
            body: None,
        }
    }
}

pub trait RawDataMode: Into<FetchArgs> {
    fn text_mode(self) -> TextMode {
        TextMode(self.into())
    }

    fn binary_mode(self) -> BinaryMode {
        BinaryMode(self.into())
    }
}
impl RawDataMode for http::request::Builder {}
impl RawDataMode for FetchArgs {}

pub struct TextMode(FetchArgs);
impl TextMode {
    pub fn body(self) -> TextBodySetter {
        TextBodySetter(self.0)
    }

    pub fn response(self) -> TextResponseSetter {
        TextResponseSetter(self.0.build_js_fetch_promise())
    }
}

pub struct BinaryMode(FetchArgs);
impl BinaryMode {
    pub fn body(self) -> BinaryBodySetter {
        BinaryBodySetter(self.0)
    }

    pub fn response(self) -> BinaryResponseSetter {
        BinaryResponseSetter(self.0.build_js_fetch_promise())
    }
}

pub struct TextBodySetter(FetchArgs);
impl TextBodySetter {
    pub fn json<T: serde::Serialize>(mut self, data: &T) -> TextBody {
        self.0.set_body(
            http::HeaderValue::from_static("application/json"),
            serde_json::to_string(&data)
                .map(From::from)
                .map_err(FetchError::EncodeJsonError),
        );
        TextBody(self.0)
    }

    pub fn text(mut self, data: &str) -> TextBody {
        self.0.set_body(
            http::HeaderValue::from_static("text/plain;charset=utf-8"),
            Ok(data.into()),
        );
        TextBody(self.0)
    }
}

pub struct BinaryBodySetter(FetchArgs);
impl BinaryBodySetter {
    pub fn json<T>(mut self, data: &T) -> BinaryBody
    where
        T: 'static + serde::Serialize,
    {
        // How about setting content type for the headers? and then just use self.0.set_body() similar to TextBody
        self.0.body = Some(
            serde_json::to_vec(data)
                .map(|data| js_sys::Uint8Array::from(data.as_slice()).into())
                .map_err(FetchError::EncodeJsonError),
        );
        BinaryBody(self.0)
    }
}

pub struct TextBody(FetchArgs);
impl TextBody {
    pub fn response(self) -> TextResponseSetter {
        TextResponseSetter(self.0.build_js_fetch_promise())
    }

    #[deprecated(
        since = "0.0.5",
        note = "Replaced by request.text_mode().response().json() or request.text_mode().body().json().response().json()"
    )]
    pub fn json_response<C, T, Cl>(
        self,
        ok_handler: fn(&mut C, T) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, RawTextForJson, T, Cl>>
    where
        C: crate::component::Component,
        T: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        self.response().json(ok_handler, error_handler)
    }
}

pub struct BinaryBody(FetchArgs);
impl BinaryBody {
    pub fn response(self) -> BinaryResponseSetter {
        BinaryResponseSetter(self.0.build_js_fetch_promise())
    }
}

impl FetchArgs {
    fn set_body(
        &mut self,
        content_type: http::HeaderValue,
        body: Result<wasm_bindgen::JsValue, FetchError>,
    ) {
        if let Some(headers) = self.request_builder.headers_mut() {
            headers.insert(http::header::CONTENT_TYPE, content_type);
            self.body = Some(body);
        }
    }

    fn build_js_fetch_promise(self) -> Result<js_sys::Promise, FetchError> {
        let body = self.body.transpose()?;
        let parts = self.request_builder.body(())?.into_parts().0;
        let ws_request = build_request(parts, body.as_ref())?;
        let init = self.options.unwrap_or_else(Default::default).into();
        let promise = crate::utils::window().fetch_with_request_and_init(&ws_request, &init);
        Ok(promise)
    }

    #[deprecated(
        since = "0.0.5",
        note = "Replaced by request.text_mode().body().json()"
    )]
    pub fn json_body<B: serde::Serialize>(self, data: &B) -> TextBody {
        TextBodySetter(self).json(data)
    }

    #[deprecated(
        since = "0.0.5",
        note = "Replaced by request.text_mode().response().json() or request.text_mode().body().json().response().json()"
    )]
    pub fn json_response<C, T, Cl>(
        self,
        ok_handler: fn(&mut C, T) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, RawTextForJson, T, Cl>>
    where
        C: crate::component::Component,
        T: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        TextResponseSetter(self.build_js_fetch_promise()).json(ok_handler, error_handler)
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

pub struct TextResponseSetter(Result<js_sys::Promise, FetchError>);
pub struct BinaryResponseSetter(Result<js_sys::Promise, FetchError>);

impl TextResponseSetter {
    pub fn json<C, T, Cl>(
        self,
        ok_handler: fn(&mut C, T) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, RawTextForJson, T, Cl>>
    where
        C: crate::component::Component,
        T: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        FetchCmdArgs {
            phantom: std::marker::PhantomData,
            promise: self.0,
            ok_handler,
            error_handler,
        }
        .into()
    }

    pub fn text<C, Cl>(
        self,
        ok_handler: fn(&mut C, String) -> Cl,
        error_handler: fn(&mut C, crate::FetchError),
    ) -> Box<FetchCmd<C, String, String, Cl>>
    where
        C: crate::component::Component,
        Cl: 'static + Into<crate::component::Checklist<C>>,
    {
        FetchCmdArgs {
            phantom: std::marker::PhantomData,
            promise: self.0,
            ok_handler,
            error_handler,
        }
        .into()
    }
}

pub trait RawData: Sized {
    fn get_raw_js(response: web_sys::Response) -> Result<js_sys::Promise, FetchError>;
    fn map_js_to_raw_data(js_value: wasm_bindgen::JsValue) -> Result<Self, FetchError>;

    // Not supported yet, so implement as a generic function in next lines
    // async fn get_raw_data(response: web_sys::Response) -> Result<Self, FetchError> { }
}

async fn get_raw_data_from_successful_response<R: RawData>(
    response: web_sys::Response,
) -> Result<R, FetchError> {
    let promise = R::get_raw_js(response)?;
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| FetchError::InvalidResponse)
        .and_then(R::map_js_to_raw_data)
}

async fn get_raw_data<R: RawData>(promise: js_sys::Promise) -> Result<R, FetchError> {
    let sr = get_successful_response(promise).await?;
    get_raw_data_from_successful_response(sr).await
}

impl RawData for String {
    fn get_raw_js(response: web_sys::Response) -> Result<js_sys::Promise, FetchError> {
        response.text().map_err(|_| FetchError::InvalidResponse)
    }

    fn map_js_to_raw_data(js_value: wasm_bindgen::JsValue) -> Result<Self, FetchError> {
        js_value.as_string().ok_or_else(|| FetchError::NotAString)
    }
}

pub struct RawTextForJson(String);
impl RawData for RawTextForJson {
    fn get_raw_js(response: web_sys::Response) -> Result<js_sys::Promise, FetchError> {
        String::get_raw_js(response)
    }

    fn map_js_to_raw_data(js_value: wasm_bindgen::JsValue) -> Result<Self, FetchError> {
        String::map_js_to_raw_data(js_value).map(Self)
    }
}

impl RawData for Vec<u8> {
    fn get_raw_js(response: web_sys::Response) -> Result<js_sys::Promise, FetchError> {
        response
            .array_buffer()
            .map_err(|_| FetchError::InvalidResponse)
    }

    fn map_js_to_raw_data(js_value: wasm_bindgen::JsValue) -> Result<Self, FetchError> {
        Ok(js_sys::Uint8Array::new(&js_value).to_vec())
    }
}

pub struct RawBinaryForJson(Vec<u8>);
impl RawData for RawBinaryForJson {
    fn get_raw_js(response: web_sys::Response) -> Result<js_sys::Promise, FetchError> {
        Vec::<u8>::get_raw_js(response)
    }

    fn map_js_to_raw_data(js_value: wasm_bindgen::JsValue) -> Result<Self, FetchError> {
        Vec::<u8>::map_js_to_raw_data(js_value).map(Self)
    }
}

struct FetchCmdArgs<C, R, T, Cl> {
    phantom: std::marker::PhantomData<R>,
    promise: Result<js_sys::Promise, FetchError>,
    ok_handler: fn(&mut C, T) -> Cl,
    error_handler: fn(&mut C, FetchError),
}

impl<C, R, T, Cl> From<FetchCmdArgs<C, R, T, Cl>> for Box<FetchCmd<C, R, T, Cl>> {
    fn from(fca: FetchCmdArgs<C, R, T, Cl>) -> Self {
        Box::new(FetchCmd(Some(fca)))
    }
}

pub struct FetchCmd<C, R, T, Cl>(Option<FetchCmdArgs<C, R, T, Cl>>);
impl<C, R, T, Cl> crate::component::Command<C> for FetchCmd<C, R, T, Cl>
where
    C: 'static + crate::component::Component,
    R: 'static + RawData,
    T: 'static + ParseFrom<R>,
    Cl: 'static + Into<crate::component::Checklist<C>>,
{
    fn execute(&mut self, comp: &crate::component::Comp<C>, state: &mut C) {
        let FetchCmdArgs {
            phantom: _,
            promise,
            ok_handler,
            error_handler,
        } = self
            .0
            .take()
            .expect_throw("Internal error: Why FetchCmd is executed twice?");
        let promise = match promise {
            Ok(promise) => promise,
            Err(e) => {
                error_handler(state, e);
                return;
            }
        };

        let error_handler = comp.callback_arg(error_handler);
        let ok_handler = comp.callback_arg(ok_handler);
        let f = async move {
            match get_raw_data::<R>(promise).await.and_then(T::parse_from) {
                Ok(data) => ok_handler(data),
                Err(e) => error_handler(e),
            }
        };
        wasm_bindgen_futures::spawn_local(f);
    }
}

async fn get_successful_response(
    promise: js_sys::Promise,
) -> Result<web_sys::Response, FetchError> {
    let response = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| FetchError::FetchFailed)?;

    let response = web_sys::Response::from(response);
    let status = http::StatusCode::from_u16(response.status())?;
    if !status.is_success() {
        return Err(FetchError::ResponseWithError(status));
    }

    Ok(response)
}

// Unable to use std::convert::TryFrom because of foreign trait/type restriction
pub trait ParseFrom<R>: Sized {
    fn parse_from(r: R) -> Result<Self, FetchError>;
}

impl ParseFrom<String> for String {
    fn parse_from(s: String) -> Result<String, FetchError> {
        Ok(s)
    }
}

impl<T> ParseFrom<RawTextForJson> for T
where
    T: serde::de::DeserializeOwned,
{
    fn parse_from(r: RawTextForJson) -> Result<T, FetchError> {
        serde_json::from_str::<T>(&r.0).map_err(FetchError::ParseJsonError)
    }
}

impl<T> ParseFrom<RawBinaryForJson> for T
where
    T: serde::de::DeserializeOwned,
{
    fn parse_from(r: RawBinaryForJson) -> Result<T, FetchError> {
        serde_json::from_slice::<T>(&r.0).map_err(FetchError::ParseJsonError)
    }
}
