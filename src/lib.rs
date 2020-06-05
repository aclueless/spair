#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod application;
mod component;
mod dom;
mod events;
mod fetch;
mod renderable;
mod routing;
mod utils;

pub use component::{
    update_component, Checklist, ChildComp, Comp, Component, Context, ShouldRender, WithParentComp,
};
pub use dom::attribute_types::*;
#[cfg(feature = "keyed-list")]
pub use dom::KeyedListItem;
pub use dom::{ElementUpdater as Element, ListElementCreation, Nodes, RawWrapper, StaticNodes};
// TODO selectively export event traits only?
pub use events::*;
pub use fetch::{FetchArgs, FetchError, Request};
pub use renderable::*;
pub use routing::Routes;
pub use utils::{document, into_input, into_select, window};

pub use web_sys::Location;
pub type Command<C> = Box<dyn component::Command<C>>;

pub mod prelude {
    pub use crate::application::Application;
    pub use crate::component::Component;
    pub use crate::dom::{AttributeSetter, DomBuilder};
    pub use crate::fetch::IntoFetchArgs;
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::UnwrapThrowExt;
}
