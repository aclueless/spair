#[cfg(feature = "keyed-list")]
mod peekable_double_ended_iterator;
#[cfg(feature = "keyed-list")]
pub use peekable_double_ended_iterator::*;

use wasm_bindgen::UnwrapThrowExt;

pub fn window() -> web_sys::Window {
    web_sys::window().expect_throw("Unable to get window")
}

pub fn document() -> web_sys::Document {
    window().document().expect_throw("Unable to get document")
}
