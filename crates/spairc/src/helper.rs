use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Element, Event, EventTarget, HtmlElement, HtmlSelectElement};

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

pub trait ElementFromCurrentEventTarget {
    fn current_target(&self) -> EventTarget;
    fn current_target_as_select(&self) -> HtmlSelectElement {
        self.current_target().unchecked_into()
    }
}

impl ElementFromCurrentEventTarget for Event {
    fn current_target(&self) -> EventTarget {
        self.current_target().unwrap_throw()
    }
}
