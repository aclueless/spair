// This module has traits that provide methods for HTML elements and HTML
// attributes. But the trait's names are too long, so their names are
// shorten with prefixes as `Hems` and `Hams`.
// `Hems` is short for `HTML element methods`
// `Hams` is short for `HTML attribute methods`

use crate::dom::NameSpace;

mod attributes;
mod attributes_elements_with_ambiguous_names;
mod attributes_with_predefined_values;
mod element;
mod list;
mod nodes;
mod render;

pub use attributes::*;
pub use attributes_elements_with_ambiguous_names::*;
pub use attributes_with_predefined_values::*;
pub use element::*;
pub use list::*;
pub use nodes::*;
pub use render::*;

struct HtmlNameSpace;
impl NameSpace for HtmlNameSpace {
    const NAMESPACE: Option<&'static str> = None;
}

// This is a struct to make sure that a name that appears in both
// HTML element names and HTML attribute names causes a conflict
// and fail to compile (during test).
#[cfg(test)]
pub struct TestHtmlMethods;
