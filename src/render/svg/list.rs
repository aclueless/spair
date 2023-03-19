use crate::{
    component::Component,
    render::{
        base::{ElementUpdaterMut, MakeNodesExtensions, NodesExtensions, NodesUpdater},
        svg::{SvgAttributesOnly, SvgElementUpdater, SvgStaticAttributes, SvgStaticAttributesOnly},
        ListElementCreation,
    },
};

pub trait SemsForList<'a, C: Component>:
    Sized + ElementUpdaterMut<'a, C> + MakeNodesExtensions<'a>
{
    fn list<I, II, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        render: R,
    ) -> NodesExtensions<'a>
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::SvgNodes<C>),
    {
        let (comp, state, mut r) = self.element_updater_mut().list_updater(mode);
        let _do_we_have_to_care_about_this_returned_value_ =
            r.render(comp, state, items, |item: I, mut nodes: NodesUpdater<C>| {
                render(item, crate::SvgNodes::new(&mut nodes))
            });
        self.make_nodes_extensions()
    }

    fn list_clone<I, II, R>(self, items: II, render: R) -> NodesExtensions<'a>
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::SvgNodes<C>),
    {
        self.list(items, ListElementCreation::Clone, render)
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgElementUpdater<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgStaticAttributes<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgStaticAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> SemsForList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
