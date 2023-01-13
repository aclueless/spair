use super::{SvgAttributesOnly, SvgStaticAttributes, SvgStaticAttributesOnly};
use crate::{
    component::{Comp, Component},
    dom::WsElement,
    render::base::{ElementUpdater, ElementUpdaterMut},
};

pub struct SvgElementUpdater<'updater, C: Component>(ElementUpdater<'updater, C>);

impl<'updater, C: Component> From<ElementUpdater<'updater, C>> for SvgElementUpdater<'updater, C> {
    fn from(element_updater: ElementUpdater<'updater, C>) -> Self {
        Self(element_updater)
    }
}

impl<'updater, C: Component> ElementUpdaterMut<'updater, C> for SvgElementUpdater<'updater, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.0
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<'updater, C> {
        &mut self.0
    }
}

impl<'updater, C: Component> SvgElementUpdater<'updater, C> {
    pub(super) fn into_inner(self) -> ElementUpdater<'updater, C> {
        self.0
    }

    pub fn state(&self) -> &'updater C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }

    pub fn attributes_only(self) -> SvgAttributesOnly<'updater, C> {
        SvgAttributesOnly::new(self.0)
    }

    pub fn static_attributes_only(self) -> SvgStaticAttributesOnly<'updater, C> {
        SvgStaticAttributesOnly::new(self.0)
    }

    pub fn static_attributes(self) -> SvgStaticAttributes<'updater, C> {
        SvgStaticAttributes::new(self.0)
    }

    pub fn ws_element(&self) -> &WsElement {
        self.0.element().ws_element()
    }
}
