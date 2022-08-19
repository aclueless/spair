use crate::dom::NameSpace;

mod attributes;
mod attributes_elements_with_ambiguous_names;
mod element;
#[cfg(feature = "keyed-list")]
mod keyed_list;
mod list;
mod nodes;
mod partial_list;
mod render;

pub use attributes::*;
pub use attributes_elements_with_ambiguous_names::*;
pub use element::*;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;
pub use list::*;
pub use nodes::*;
pub use partial_list::*;
pub use render::*;

pub struct SvgNameSpace;
impl NameSpace for SvgNameSpace {
    const NAMESPACE: &'static str = "http://www.w3.org/2000/svg";
}

// This is a struct to make sure that a name that appears in both
// SVG element names and SVG attribute names causes a conflict
// and fail to compile (during test).
#[cfg(test)]
pub struct TestSvgMethods;
