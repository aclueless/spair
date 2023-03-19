use crate::{
    component::Component,
    render::{
        base::{ElementUpdaterMut, MakeNodesExtensions, NodesExtensions, NodesUpdater},
        html::{
            AttributesOnly, HtmlElementUpdater, HtmlNodesUpdater, StaticAttributes,
            StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForList<'a, C: Component>:
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
        R: Fn(I, crate::Nodes<C>),
    {
        let (comp, state, mut r) = self.element_updater_mut().list_updater(mode);
        let _do_we_have_to_care_about_this_returned_value_ =
            r.render(comp, state, items, |item: I, nodes: NodesUpdater<C>| {
                let mut nodes = HtmlNodesUpdater::new(nodes);
                render(item, crate::Nodes::new(&mut nodes))
            });

        self.make_nodes_extensions()
    }

    fn list_clone<I, II, R>(self, items: II, render: R) -> NodesExtensions<'a>
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::Nodes<C>),
    {
        self.list(items, ListElementCreation::Clone, render)
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for HtmlElementUpdater<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for AttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for StaticAttributes<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for StaticAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> HemsForList<'a, C> for HtmlElementUpdater<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for StaticAttributesOnly<'a, C> {}
