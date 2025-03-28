#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod elements;
mod events;
mod keyed_list;
mod routing;

mod helper;

#[cfg(test)]
mod test_helper;

pub use wasm_bindgen;
pub use web_sys;

pub use component::{start_app, start_app_with_routing, CallbackArg, Comp, Component, Context};
pub use elements::{Element, TemplateElement, Text, WsElement, WsNode, WsText};
pub use keyed_list::{KeyedItemView, KeyedList};
pub use routing::Route;

pub mod prelude {
    pub use crate::component::{CallbackArg, Comp, Context};
    // pub use crate::elements::{Element, TemplateElement, Text, WsElement, WsNode, WsText};
    // pub use crate::{KeyedItemView, KeyedList};
    pub use spair_macros::*;
}
