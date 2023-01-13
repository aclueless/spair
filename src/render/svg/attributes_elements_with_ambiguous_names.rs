use super::{
    SvgAttributesOnly, SvgElementUpdater, SvgNodes, SvgNodesOwned, SvgStaticAttributes,
    SvgStaticAttributesOnly, SvgStaticNodes, SvgStaticNodesOwned, UpdateSvgElement,
};
use crate::{
    component::Component,
    render::base::{NodesUpdaterMut, StringAttributeValue},
};

#[cfg(test)]
use crate::render::svg::TestSvgMethods;

make_trait_for_same_name_attribute_and_element_methods! {
    TestStructs: (TestSvgMethods)
    DeprecatedTraitName: SemsSamsAmbiguous
    for_elements {
        TraitName: SemsForAmbiguousNames
        UpdateElementTraitName: UpdateSvgElement
        ElementUpdaterType: SvgElementUpdater
    }
    for_attributes {
        TraitName: SamsForAmbiguousNames
    }
    ambiguous_attributes:
    // The names are also used to make methods for SVG elements
    //  type    name
        str     clip_path :a:"clip-path" :e:"clipPath"
        str     mask
        str     path
}

impl<'er, C: Component> SemsSamsAmbiguous for SvgElementUpdater<'er, C> {}
impl<'er, C: Component> SemsSamsAmbiguous for SvgStaticAttributes<'er, C> {}

impl<'er, C: Component> SamsForAmbiguousNames<'er, C> for SvgAttributesOnly<'er, C> {}
impl<'er, C: Component> SamsForAmbiguousNames<'er, C> for SvgStaticAttributesOnly<'er, C> {}

impl<'n, C: Component> SemsForAmbiguousNames<'n, C> for SvgStaticNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsForAmbiguousNames<'n, C> for SvgNodesOwned<'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> SemsForAmbiguousNames<'n, C> for SvgStaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> SemsForAmbiguousNames<'n, C> for SvgNodes<'h, 'n, C> {
    type Output = Self;
}
