use super::{HtmlElementRender, RenderHtmlElement};
use crate::component::Component;
use crate::render::base::{
    ElementRenderMut, NodeListRenderMut, StringAttributeValue, U32AttributeValue,
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
