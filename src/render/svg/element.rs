use super::{
    SamsForDistinctNames, SamsHandMade, SemsForDistinctNames, SvgAttributesOnly, SvgNodesOwned,
    SvgStaticAttributes, SvgStaticAttributesOnly,
};
use crate::component::{Comp, Component};
use crate::render::base::{ElementRender, ElementRenderMut};

pub struct SvgElementRender<'er, C: Component>(ElementRender<'er, C>);

impl<'er, C: Component> From<ElementRender<'er, C>> for SvgElementRender<'er, C> {
    fn from(element_render: ElementRender<'er, C>) -> Self {
        Self(element_render)
    }
}

impl<'er, C: Component> ElementRenderMut<C> for SvgElementRender<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        &self.0
    }
    fn element_render_mut(&mut self) -> &'er mut ElementRender<C> {
        &mut self.0
    }
}

impl<'er, C: Component> SvgElementRender<'er, C> {
    pub(super) fn into_inner(self) -> ElementRender<'er, C> {
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

    pub fn ws_element(&self) -> &web_sys::Element {
        self.0.element().ws_element()
    }
}

impl<'er, C: Component> SamsForDistinctNames<C> for SvgElementRender<'er, C> {}
impl<'er, C: Component> SamsHandMade<C> for SvgElementRender<'er, C> {}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgElementRender<'n, C> {
    type Output = SvgNodesOwned<'n, C>;
}
