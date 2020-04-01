pub mod application;
mod component;
mod dom;
mod events;
mod fetch;
mod renderable;
mod routing;
mod utils;

pub use component::*;
pub use dom::attribute_types::*;
pub use dom::{ElementHandle as Element, Nodes, StaticNodes};
pub use events::*;
pub use fetch::*;
pub use renderable::*;
pub use routing::Routes;
pub use utils::*;

pub use web_sys::Location;

pub mod prelude {
    pub use crate::dom::{AttributeSetter, DomBuilder};
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::UnwrapThrowExt;
}
