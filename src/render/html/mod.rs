// This module has traits that provide methods for HTML elements and HTML
// attributes. But the trait's names are too long, so their names are
// shorten with prefixes as `Hems` and `Hams`.
// `Hems` is short for `HTML element methods`
// `Hams` is short for `HTML attribute methods`

use crate::dom::ElementTag;

mod attributes;
mod attributes_elements_with_ambiguous_names;
mod attributes_with_predefined_values;
mod element;
mod list;
mod nodes;
mod partial_list;
mod render;

pub use attributes::*;
pub use attributes_elements_with_ambiguous_names::*;
pub use attributes_with_predefined_values::*;
pub use element::*;
pub use list::*;
pub use nodes::*;
pub use partial_list::*;
pub use render::*;

#[cfg(feature = "keyed-list")]
mod keyed_list;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;

#[derive(Copy, Clone)]
pub struct HtmlTag(pub &'static str);

impl ElementTag for HtmlTag {
    const NAMESPACE: &'static str = "http://www.w3.org/1999/xhtml";
    fn tag_name(&self) -> &str {
        self.0
    }
}

// This is a struct to make sure that a name that appears in both
// HTML element names and HTML attribute names causes a conflict
// and fail to compile (during test).
#[cfg(test)]
pub struct TestHtmlMethods;
