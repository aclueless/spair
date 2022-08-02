use crate::component::Component;
use crate::render::base::{ElementRender, ElementRenderMut};
use super::{SamsHandMade, SamsForDistinctNames, SemsForDistinctNames, SvgNodesOwned};

pub struct SvgElementRender<'er, C: Component>(ElementRender<'er, C>);

impl<'er, C: Component> From<ElementRender<'er, C>> for SvgElementRender<'er, C> {
    fn from(element_render: ElementRender<'er, C>) -> Self {
        Self(element_render)
    }
}

impl<'er, C: Component> ElementRenderMut<C> for SvgElementRender<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        self.element_render()
    }
    fn element_render_mut(&mut self) -> &mut ElementRender<C> {
        self.element_render_mut()
    }
}

impl<'er, C: Component> SvgElementRender<'er, C> {
    pub(super) fn into_inner(self) -> ElementRender<'er, C> {
        self.0
    }
}

impl<'er, C: Component> SamsForDistinctNames<C> for SvgElementRender<'er, C> {}
impl<'er, C: Component> SamsHandMade<C> for SvgElementRender<'er, C> {}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgElementRender<'n, C> {
    type Output = SvgNodesOwned<'n, C>;
}
