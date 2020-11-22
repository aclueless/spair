pub trait SvgRender<C: crate::component::Component> {
    fn render(self, nodes: super::SvgNodes<C>);
}

pub trait SvgStaticRender<C: crate::component::Component> {
    fn render(self, nodes: super::SvgStaticNodes<C>);
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: crate::component::Component> SvgRender<C> for $type {
                fn render(self, nodes: super::SvgNodes<C>) {
                    nodes.update_text(&self.to_string());
                }
            }

            impl<C: crate::component::Component> SvgStaticRender<C> for $type {
                fn render(self, nodes: super::SvgStaticNodes<C>) {
                    nodes.static_text(&self.to_string());
                }
            }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool char
}

impl<C: crate::component::Component> SvgRender<C> for &str {
    fn render(self, nodes: super::SvgNodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: crate::component::Component> SvgStaticRender<C> for &str {
    fn render(self, nodes: super::SvgStaticNodes<C>) {
        nodes.static_text(self);
    }
}
