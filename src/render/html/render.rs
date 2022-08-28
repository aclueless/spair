use super::{Nodes, StaticNodes};
use crate::component::Component;
/// This module provides traits that help users define how their types should be rendered.
/// Implementation for primitive types are also provided.

pub trait Render<C: Component> {
    fn render(self, nodes: Nodes<C>);
}

pub trait StaticRender<C: Component> {
    fn render(self, nodes: StaticNodes<C>);
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: Component> Render<C> for $type {
                fn render(self, nodes: Nodes<C>) {
                    nodes.update_text(&self.to_string());
                }
            }

            impl<C: Component> StaticRender<C> for $type {
                fn render(self, nodes: StaticNodes<C>) {
                    nodes.static_text(&self.to_string());
                }
            }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool char
}

impl<C: Component> StaticRender<C> for &str {
    fn render(self, nodes: StaticNodes<C>) {
        nodes.static_text(self);
    }
}

impl<C: Component> StaticRender<C> for &String {
    fn render(self, nodes: StaticNodes<C>) {
        nodes.static_text(self);
    }
}

impl<C: Component> StaticRender<C> for String {
    fn render(self, nodes: StaticNodes<C>) {
        nodes.static_text(&self);
    }
}

impl<C: Component> Render<C> for &str {
    fn render(self, nodes: Nodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: Component> Render<C> for &String {
    fn render(self, nodes: Nodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: Component> Render<C> for String {
    fn render(self, nodes: Nodes<C>) {
        nodes.update_text(&self);
    }
}

pub trait ListItemRender<C: Component> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(self, item: crate::Element<C>);
}

pub trait ListItemRenderRef<C: Component> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(&self, item: crate::Element<C>);
}
