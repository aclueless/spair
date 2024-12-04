#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod elements;
mod events;
mod keyed_list;

pub use wasm_bindgen;
pub use web_sys;

pub use component::{start_app, CallbackArg, Comp, Component, ComponentRoot, Context};
pub use elements::{Element, TemplateElement, WsElement};
pub use keyed_list::{KeyedItemRender, KeyedList};
