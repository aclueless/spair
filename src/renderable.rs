/// This module provides traits that help users define how their types should be rendered.
/// Implementation for primitive types is also provided.
use crate::dom::{Nodes, StaticNodes};

pub trait Render<C: crate::component::Component> {
    fn render(self, nodes: Nodes<C>) -> Nodes<C>;
}

pub trait StaticRender<C: crate::component::Component> {
    fn render(self, nodes: StaticNodes<C>) -> StaticNodes<C>;
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: crate::component::Component> Render<C> for $type {
                fn render(self, nodes: Nodes<C>) -> Nodes<C> {
                    nodes.update_text(&self.to_string())
                }
            }
            impl<C: crate::component::Component> StaticRender<C> for $type {
                fn render(self, nodes: StaticNodes<C>) -> StaticNodes<C> {
                    nodes.static_text(&self.to_string())
                }
            }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool
}

// Special case for 'static str => always render as static text
impl<C: crate::component::Component> Render<C> for &'static str {
    fn render(self, nodes: Nodes<C>) -> Nodes<C> {
        nodes.static_text(self)
    }
}

impl<C: crate::component::Component> StaticRender<C> for &str {
    fn render(self, nodes: StaticNodes<C>) -> StaticNodes<C> {
        nodes.static_text(self)
    }
}

impl<C: crate::component::Component> Render<C> for &String {
    fn render(self, nodes: Nodes<C>) -> Nodes<C> {
        nodes.update_text(self)
    }
}

impl<C: crate::component::Component> StaticRender<C> for &String {
    fn render(self, nodes: StaticNodes<C>) -> StaticNodes<C> {
        nodes.static_text(self)
    }
}

pub trait ListItem<C: crate::component::Component> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(&self, state: &C, item: crate::dom::ElementUpdater<C>);
}
