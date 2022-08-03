use super::{
    AttributesOnly, HtmlElementRender, Nodes, NodesOwned, RenderHtmlElement, StaticAttributes,
    StaticAttributesOnly, StaticNodes, StaticNodesOwned,
};
use crate::component::Component;
use crate::render::base::{
    ElementRenderMut, NodesRenderMut, StringAttributeValue, U32AttributeValue,
};

#[cfg(test)]
use crate::render::html::TestHtmlMethods;

make_trait_for_same_name_attribute_and_element_methods! {
    TestStructs: (TestHtmlMethods)
    DeprecatedTraitName: HemsHamsAmbiguous
    for_elements {
        TraitName: HemsForAmbiguousNames
        RenderElementTraitName: RenderHtmlElement
        ElementRenderType: HtmlElementRender
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

impl<'er, C: Component> HemsHamsAmbiguous for HtmlElementRender<'er, C> {}
impl<'er, C: Component> HemsHamsAmbiguous for StaticAttributes<'er, C> {}

impl<'er, C: Component> HamsForAmbiguousNames<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> HamsForAmbiguousNames<C> for StaticAttributesOnly<'er, C> {}

impl<'n, C: Component> HemsForAmbiguousNames<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForAmbiguousNames<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForAmbiguousNames<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForAmbiguousNames<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
