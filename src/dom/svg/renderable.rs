pub trait SvgRender<C: crate::component::Component> {
    fn render(self, nodes: super::SvgNodes<C>);
}

pub trait SvgStaticRender<C: crate::component::Component> {
    fn render(self, nodes: super::SvgStaticNodes<C>);
}

