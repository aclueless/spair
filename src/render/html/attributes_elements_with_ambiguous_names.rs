use super::{HtmlElementRender, RenderHtmlElement};
use crate::component::Component;
use crate::render::base::{ElementRenderMut, NodeListRenderMut, U32AttributeValue};

#[cfg(test)]
use super::AllAttributes;
#[cfg(test)]
use super::AllElements;

make_trait_for_same_name_attribute_and_element_methods! {
    TestStructs: (AllElements AllAttributes)
    DeprecatedTraitName: MethodsForDeprecatingAttributesAndElementsWithAmbiguousNames
    for_elements {
        TraitName: AmbiguousHtmlElementMethods
        RenderElementTraitName: RenderHtmlElement
        ElementRenderType: HtmlElementRender
    }
    for_attributes {
        TraitName: MethodsForHtmlAttributesWithAmbiguousNames
    }
    ambiguous_attributes:
        u32 span
}
