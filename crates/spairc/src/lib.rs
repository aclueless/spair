#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod element;
mod events;
mod keyed_list;
mod list;
mod routing;

mod helper;

#[cfg(test)]
mod test_helper;

pub use wasm_bindgen;
pub use web_sys;

pub use component::{start_app, start_app_with_routing, CallbackArg, Comp, Component, Context};
pub use element::{Element, TemplateElement, Text, WsElement, WsNode, WsText};
pub use keyed_list::{KeyedList, KeyedListItemView};
pub use list::{List, ListItemView};
pub use routing::Route;

pub mod prelude {
    pub use crate::component::{CallbackArg, Comp, Context};
    pub use crate::element::RenderOptionWithDefault;
    pub use spair_macros::*;
}
