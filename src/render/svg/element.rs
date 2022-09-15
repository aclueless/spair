use super::{SvgAttributesOnly, SvgStaticAttributes, SvgStaticAttributesOnly};
use crate::{
    component::{Comp, Component},
    dom::WsElement,
    render::base::{BaseElementRender, BaseElementRenderMut},
};

pub struct SvgElementRender<'er, C: Component>(BaseElementRender<'er, C>);

impl<'er, C: Component> From<BaseElementRender<'er, C>> for SvgElementRender<'er, C> {
    fn from(element_render: BaseElementRender<'er, C>) -> Self {
        Self(element_render)
    }
}

impl<'er, C: Component> BaseElementRenderMut<C> for SvgElementRender<'er, C> {
    fn element_render(&self) -> &BaseElementRender<C> {
        &self.0
    }
    fn element_render_mut(&mut self) -> &'er mut BaseElementRender<C> {
        &mut self.0
    }
}

impl<'er, C: Component> SvgElementRender<'er, C> {
    pub(super) fn into_inner(self) -> BaseElementRender<'er, C> {
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
