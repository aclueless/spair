/// This module provides traits that help users define how their types should be rendered.
/// Implementation for primitive types are also provided.

pub trait Render<C: crate::component::Component> {
    fn render(self, nodes: crate::dom::Nodes<C>);
}

pub trait StaticRender<C: crate::component::Component> {
    fn render(self, nodes: crate::dom::StaticNodes<C>);
}

pub trait RenderRef<C: crate::component::Component> {
    fn render(&self, nodes: crate::dom::Nodes<C>);
}

impl<C, T> RenderRef<C> for T
where
    C: crate::component::Component,
    for<'t> &'t T: Render<C>,
{
    fn render(&self, nodes: crate::dom::Nodes<C>) {
        Render::render(self, nodes);
    }
}

// mod sealed {
//     pub trait ListItemStaticText {}
// }

// pub trait ListItemStaticText<C: crate::component::Component>: sealed::ListItemStaticText {
//     fn render(self, nodes: NodesOwned<C>) -> NodesOwned<C>;
// }

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: crate::component::Component> Render<C> for $type {
                fn render(self, nodes: crate::dom::Nodes<C>) {
                    nodes.update_text(&self.to_string());
                }
            }

            impl<C: crate::component::Component> StaticRender<C> for $type {
                fn render(self, nodes: crate::dom::StaticNodes<C>) {
                    nodes.static_text(&self.to_string());
                }
            }

            // impl sealed::ListItemStaticText for $type {}
            // impl<C: crate::component::Component> ListItemStaticText<C> for $type {
            //     fn render(self, nodes: NodesOwned<C>) -> NodesOwned<C> {
            //         nodes.update_text(&self.to_string())
            //     }
            // }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool char
}

impl<C: crate::component::Component> StaticRender<C> for &str {
    fn render(self, nodes: crate::dom::StaticNodes<C>) {
        nodes.static_text(self);
    }
}

impl<C: crate::component::Component> StaticRender<C> for &String {
    fn render(self, nodes: crate::dom::StaticNodes<C>) {
        nodes.static_text(self);
    }
}

impl<C: crate::component::Component> Render<C> for &str {
    fn render(self, nodes: crate::dom::Nodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: crate::component::Component> Render<C> for &String {
    fn render(self, nodes: crate::dom::Nodes<C>) {
        nodes.update_text(self);
    }
}

// impl sealed::ListItemStaticText for &str {}
// impl<C: crate::component::Component> ListItemStaticText<C> for &str {
//     fn render(self, nodes: NodesOwned<C>) -> NodesOwned<C> {
//         nodes.update_text(self)
//     }
// }

// impl sealed::ListItemStaticText for &String {}
// impl<C: crate::component::Component> ListItemStaticText<C> for &String {
//     fn render(self, nodes: NodesOwned<C>) -> NodesOwned<C> {
//         nodes.update_text(self)
//     }
// }

pub trait ListItem<C: crate::component::Component> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(&self, item: crate::dom::HtmlUpdater<C>);
}

impl<C: crate::component::Component, T: ListItem<C>> ListItem<C> for &T {
    const ROOT_ELEMENT_TAG: &'static str = T::ROOT_ELEMENT_TAG;
    fn render(&self, item: crate::dom::HtmlUpdater<C>) {
        (*self).render(item);
    }
}

pub trait ListItemR<'a, C: crate::component::Component> {
    const ROOT_ELEMENT_TAG: &'static str;
    fn render(&self, item: crate::dom::HtmlUpdater<'a, C>);
}

impl<'a, C: crate::component::Component, T: ListItemR<'a, C>> ListItemR<'a, C> for &T {
    const ROOT_ELEMENT_TAG: &'static str = T::ROOT_ELEMENT_TAG;
    fn render(&self, item: crate::dom::HtmlUpdater<'a, C>) {
        (*self).render(item);
    }
}
