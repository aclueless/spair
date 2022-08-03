use crate::dom::NameSpace;

mod attributes;
mod element;
mod nodes;
mod render;
mod list;

pub use attributes::*;
pub use element::*;
pub use nodes::*;
pub use render::*;
pub use list::*;

pub struct SvgNameSpace;
impl NameSpace for SvgNameSpace {
    const NAMESPACE: Option<&'static str> = Some("http://www.w3.org/2000/svg");
}

// This is a struct to make sure that a name that appears in both
// SVG element names and SVG attribute names causes a conflict
// and fail to compile (during test).
#[cfg(test)]
pub struct TestSvgMethods;
