use super::{
    AttributesOnly, HtmlElementUpdater, Nodes, NodesOwned, StaticAttributes, StaticAttributesOnly,
    StaticNodes, StaticNodesOwned, UpdateHtmlElement,
};
use crate::{
    component::Component,
    render::base::{NodesUpdaterMut, StringAttributeValue, U32AttributeValue},
};

#[cfg(test)]
use crate::render::html::TestHtmlMethods;

make_trait_for_same_name_attribute_and_element_methods! {
    TestStructs: (TestHtmlMethods)
    DeprecatedTraitName: HemsHamsAmbiguous
    for_elements {
        TraitName: HemsForAmbiguousNames
        UpdateElementTraitName: UpdateHtmlElement
        ElementUpdaterType: HtmlElementUpdater
    }
    for_attributes {
        TraitName: HamsForAmbiguousNames
    }
    ambiguous_attributes:
    // The names are also used to make methods for HTML elements
    //  type    name
        str     abbr
        str     cite
        str     data
        str     form
        str     label
        u32     span
}

impl<'er, C: Component> HemsHamsAmbiguous for HtmlElementUpdater<'er, C> {}
impl<'er, C: Component> HemsHamsAmbiguous for StaticAttributes<'er, C> {}

impl<'er, C: Component> HamsForAmbiguousNames<'er, C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> HamsForAmbiguousNames<'er, C> for StaticAttributesOnly<'er, C> {}

impl<'n, C: Component> HemsForAmbiguousNames<'n, C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForAmbiguousNames<'n, C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForAmbiguousNames<'n, C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForAmbiguousNames<'n, C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
