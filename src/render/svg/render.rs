use super::{SvgNodes, SvgStaticNodes};
use crate::component::Component;

pub trait SvgRender<C: Component> {
    fn render(self, nodes: SvgNodes<C>);
}

pub trait SvgStaticRender<C: Component> {
    fn render(self, nodes: SvgStaticNodes<C>);
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: Component> SvgRender<C> for $type {
                fn render(self, nodes: SvgNodes<C>) {
                    nodes.update_text(&self.to_string());
                }
            }

            impl<C: Component> SvgStaticRender<C> for $type {
                fn render(self, nodes: SvgStaticNodes<C>) {
                    nodes.static_text(&self.to_string());
                }
            }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool char
}

impl<C: Component> SvgRender<C> for &str {
    fn render(self, nodes: SvgNodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: Component> SvgStaticRender<C> for &str {
    fn render(self, nodes: SvgStaticNodes<C>) {
        nodes.static_text(self);
    }
}

impl<C: Component> SvgRender<C> for &String {
    fn render(self, nodes: SvgNodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: Component> SvgStaticRender<C> for &String {
    fn render(self, nodes: SvgStaticNodes<C>) {
        nodes.static_text(self);
    }
}

pub trait SvgElementRender<C: Component> {
    const ELEMENT_TAG: &'static str;
    fn render(self, item: crate::SvgElement<C>);
}

/// A simple wrapper to render an ElementRender's item with `.rupdate()`
pub struct SvgRer<T>(T);
impl<C: Component, T: SvgElementRender<C>> SvgRender<C> for SvgRer<T> {
    fn render(self, nodes: SvgNodes<C>) {
        use super::RenderSvgElement;
        nodes.render_element(T::ELEMENT_TAG, |er| self.0.render(er));
    }
}
