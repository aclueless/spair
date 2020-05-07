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

pub use application::start;
pub use component::{Checklist, ChildComp, Comp, Component, Components, Context};
pub use dom::attribute_types::*;
#[cfg(feature = "keyed-list")]
pub use dom::KeyedListItem;
pub use dom::{ElementUpdater as Element, Nodes, RawWrapper, StaticNodes};
// TODO selectively export event traits only?
pub use events::*;
pub use fetch::{FetchError, Request};
pub use renderable::*;
pub use routing::Routes;
pub use utils::{document, into_input, window, into_select};

pub use web_sys::Location;

pub mod prelude {
    pub use crate::dom::{AttributeSetter, DomBuilder};
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::UnwrapThrowExt;
}
