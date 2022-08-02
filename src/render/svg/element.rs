use crate::component::Component;
use crate::render::base::ElementRender;

pub struct SvgElementRender<'er, C: Component>(ElementRender<'er, C>);

impl<'er, C: Component> From<ElementRender<'er, C>> for SvgElementRender<'er, C> {
    fn from(element_render: ElementRender<'er, C>) -> Self {
        Self(element_render)
    }
}

