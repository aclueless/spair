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
        str     _clip_path :a:"clip-path" :e:"clipPath"
        str     _mask
        str     _path
}

impl<'updater, C: Component> SemsSamsAmbiguous for SvgElementUpdater<'updater, C> {}
impl<'updater, C: Component> SemsSamsAmbiguous for SvgStaticAttributes<'updater, C> {}

impl<'updater, C: Component> SamsForAmbiguousNames<'updater, C> for SvgAttributesOnly<'updater, C> {}
impl<'updater, C: Component> SamsForAmbiguousNames<'updater, C>
    for SvgStaticAttributesOnly<'updater, C>
{
}

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
