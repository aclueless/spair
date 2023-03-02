use crate::dom::{ElementTag, ElementTagExt};

mod attributes;
mod attributes_elements_with_ambiguous_names;
mod element;
#[cfg(feature = "keyed-list")]
mod keyed_list;
mod list;
mod nodes;
mod partial_list;

pub use attributes::*;
pub use attributes_elements_with_ambiguous_names::*;
pub use element::*;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;
pub use list::*;
pub use nodes::*;
pub use partial_list::*;

#[derive(Copy, Clone)]
pub struct SvgTag(pub &'static str);

impl From<&'static str> for SvgTag {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

impl ElementTag for SvgTag {
    const NAMESPACE: &'static str = "http://www.w3.org/2000/svg";
    fn tag_name(&self) -> &str {
        self.0
    }
}

impl<'a, C: crate::Component> ElementTagExt<'a, C> for SvgTag {
    type Updater = SvgElementUpdater<'a, C>;
    fn make_updater(e: super::base::ElementUpdater<'a, C>) -> Self::Updater {
        e.into()
    }
}

// This is a struct to make sure that a name that appears in both
// SVG element names and SVG attribute names causes a conflict
// and fail to compile (during test).
#[cfg(test)]
pub struct TestSvgMethods;
