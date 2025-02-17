#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod elements;
mod events;
mod keyed_list;

mod helper;

mod node;
#[cfg(test)]
mod test_helper;

pub use wasm_bindgen;
pub use web_sys;

pub use component::{start_app, CallbackArg, Comp, Component, Context};
pub use elements::{Element, TemplateElement, WsElement, WsNode};
pub use keyed_list::{KeyedItemView, KeyedList};

pub mod prelude {
    pub use crate::component::{Comp, Component, Context};
    pub use crate::elements::{Element, Text, WsElement, WsText};
    pub use spair_macros::*;
}
