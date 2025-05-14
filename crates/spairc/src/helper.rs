use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{
    Element, Event, EventTarget, HtmlElement, HtmlInputElement, HtmlSelectElement, InputEvent,
};

thread_local!(
    pub static WINDOW: web_sys::Window = web_sys::window().expect_throw("No window found");
    pub static DOCUMENT: web_sys::Document =
        WINDOW.with(|window| window.document().expect_throw("No document found"));
);

#[allow(dead_code)]
pub fn get_body() -> HtmlElement {
    DOCUMENT.with(|d| d.body()).expect_throw("No body")
}

pub fn get_element_by_id(element_id: &str) -> Option<Element> {
    DOCUMENT.with(|document| document.get_element_by_id(element_id))
}

pub trait ElementFromCurrentEventTarget {
    fn get_current_target(&self) -> EventTarget;
    fn current_target_as_select(&self) -> HtmlSelectElement {
        self.get_current_target().unchecked_into()
    }
}

impl ElementFromCurrentEventTarget for Event {
    fn get_current_target(&self) -> EventTarget {
        self.current_target().unwrap_throw()
    }
}

pub trait InputElementFromCurrentInputEvent {
    fn get_current_target(&self) -> EventTarget;
    fn current_target_as_input(&self) -> HtmlInputElement {
        self.get_current_target().unchecked_into()
    }
}

impl InputElementFromCurrentInputEvent for InputEvent {
    fn get_current_target(&self) -> EventTarget {
        self.current_target().unwrap_throw()
    }
}
