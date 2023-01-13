use super::{SvgAttributesOnly, SvgStaticAttributes, SvgStaticAttributesOnly};
use crate::{
    component::{Comp, Component},
    dom::WsElement,
    render::base::{ElementUpdater, ElementUpdaterMut},
};

pub struct SvgElementUpdater<'er, C: Component>(ElementUpdater<'er, C>);

impl<'er, C: Component> From<ElementUpdater<'er, C>> for SvgElementUpdater<'er, C> {
    fn from(element_updater: ElementUpdater<'er, C>) -> Self {
        Self(element_updater)
    }
}

impl<'er, C: Component> ElementUpdaterMut<'er, C> for SvgElementUpdater<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.0
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<'er, C> {
        &mut self.0
    }
}

impl<'er, C: Component> SvgElementUpdater<'er, C> {
    pub(super) fn into_inner(self) -> ElementUpdater<'er, C> {
        self.0
    }

    pub fn state(&self) -> &'er C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }

    pub fn attributes_only(self) -> SvgAttributesOnly<'er, C> {
        SvgAttributesOnly::new(self.0)
    }

    pub fn static_attributes_only(self) -> SvgStaticAttributesOnly<'er, C> {
        SvgStaticAttributesOnly::new(self.0)
    }

    pub fn static_attributes(self) -> SvgStaticAttributes<'er, C> {
        SvgStaticAttributes::new(self.0)
    }

    pub fn ws_element(&self) -> &WsElement {
        self.0.element().ws_element()
    }
}
