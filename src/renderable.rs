use crate::dom::{Nodes, StaticNodes};

pub trait Render<C> {
    fn render<'a>(self, nodes: Nodes<'a, C>) -> Nodes<'a, C>;
}

pub trait StaticRender<C> {
    fn render<'a>(self, nodes: StaticNodes<'a, C>) -> StaticNodes<'a, C>;
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C> Render<C> for $type {
                fn render<'a>(self, nodes: Nodes<'a, C>) -> Nodes<'a, C> {

                    nodes.update_text(&self.to_string())
                }
            }
            impl<C> StaticRender<C> for $type {
                fn render<'a>(self, nodes: StaticNodes<'a, C>) -> StaticNodes<'a, C> {

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
impl<C> Render<C> for &'static str {
    fn render<'a>(self, nodes: Nodes<'a, C>) -> Nodes<'a, C> {
        nodes.static_text(self)
    }
}

impl<C> StaticRender<C> for &str {
    fn render<'a>(self, nodes: StaticNodes<'a, C>) -> StaticNodes<'a, C> {
        nodes.static_text(self)
    }
}

impl<C> Render<C> for &String {
    fn render<'a>(self, nodes: Nodes<'a, C>) -> Nodes<'a, C> {
        nodes.update_text(self)
    }
}

impl<C> StaticRender<C> for &String {
    fn render<'a>(self, nodes: StaticNodes<'a, C>) -> StaticNodes<'a, C> {
        nodes.static_text(self)
    }
}

pub trait ListItem<C> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(&self, item: crate::dom::ElementHandle<C>, state: &C);
}
