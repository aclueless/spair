use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Element, HtmlElement};

thread_local!(
    pub static WINDOW: web_sys::Window = web_sys::window().expect_throw("No window found");
    pub static DOCUMENT: web_sys::Document =
        WINDOW.with(|window| window.document().expect_throw("No document found"));
);

pub fn get_body() -> HtmlElement {
    DOCUMENT.with(|d| d.body()).expect_throw("No body")
}

pub fn get_element_by_id(element_id: &str) -> Option<Element> {
    DOCUMENT.with(|document| document.get_element_by_id(element_id))
}
