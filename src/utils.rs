use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub fn window() -> web_sys::Window {
    web_sys::window().expect_throw("Unable to get window")
}

pub fn document() -> web_sys::Document {
    window().document().expect_throw("Unable to get document")
}

pub fn into_input(et: web_sys::EventTarget) -> web_sys::HtmlInputElement {
    et.dyn_into()
        .expect_throw("Unable to convert event target to web_sys::HtmlInputElement")
}
