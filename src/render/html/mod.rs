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

// All elements must have a method on this struct.
// Attribute-only must have a method on this struct.
// If there is a conflict, consider move the element/attribute to ?
#[cfg(test)]
pub struct AllElements;

// All attributes must have a method on this struct.
// Element-only must have a method on this struct.
// If there is a conflict, consider move the element/attribute to ?
#[cfg(test)]
pub struct AllAttributes;
