#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod dom;
mod events;
mod helper;
mod keyed_list;
mod list;
mod routing;

pub use web_sys;

pub use component::{
    Callback, CallbackArg, Comp, CompNode, Component, ComponentViewState, Context, RcComp,
    start_app, start_app_with_routing,
};
pub use dom::{
    Element, TemplateElement, WsElement, WsNode, WsNodeFns,
    text::{RenderOptionWithDefault, Text, WsText},
};
pub use keyed_list::{ItemViewState, KeyedList};
pub use list::List;
pub use routing::Route;
pub use web_sys::DocumentFragment;

pub mod prelude {
    pub use crate::component::{
        Callback, CallbackArg, Comp, Context, RcComp, SpairSpawnLocal, SpairSpawnLocalWithCallback,
    };
    pub use crate::helper::{ElementFromCurrentEventTarget, InputElementFromCurrentInputEvent};
    pub use spair_macros::{create_view, impl_component};
}
