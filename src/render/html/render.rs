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

pub trait ElementRender<C: Component> {
    const ELEMENT_TAG: &'static str;
    fn render(self, item: crate::Element<C>);
}

// A simple wrapper to render an ElementRender's item with `.rupdate()`
// pub struct Rer<T>(T);
// impl<C: Component, T: ElementRender<C>> Render<C> for Rer<T> {
//     fn render(self, nodes: Nodes<C>) {
//         use super::UpdateHtmlElement;
//         nodes.render_element(T::ELEMENT_TAG, |er| self.0.render(er));
//     }
// }

// Rust prevent implementation directly on T with this error:
// error[E0119]: conflicting implementations of trait `render::html::render::Render<_>` for type `&str`
//    --> src/render/html/render.rs:100:1
//     |
// 54  | impl<C: Component> Render<C> for &str {
//     | ------------------------------------- first implementation here
// ...
// 100 | impl<C: Component, T: ElementRender<C>> Render<C> for T {
//     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `&str`
//     |
//     = note: downstream crates may implement trait `render::html::render::ElementRender<_>` for type `&str`
// TODO: Ultimate goal: Request rust lift the restriction, because the downstream crates should not
//       impl `render::html::render::ElementRender<_>` for type `&str`
// TODO: Near future todo: Make a proc macro #[derive(RenderWithElementRender)]
// impl<C: Component, T: ElementRender<C>> Render<C> for T {
//     fn render(self, nodes: Nodes<C>) {
//         use super::UpdateHtmlElement;
//         nodes.render_element(T::ELEMENT_TAG, |er| self.render(er));
//     }
// }

// Similarly to Svg renders
