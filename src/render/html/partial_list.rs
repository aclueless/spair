use super::{Nodes, NodesOwned, StaticNodes, StaticNodesOwned};
use crate::{
    component::Component,
    render::{
        base::{NodesUpdater, NodesUpdaterMut},
        html::HtmlNodesUpdater,
        ListElementCreation,
    },
};

pub trait HemsForPartialList<'a, C: Component>: Sized + NodesUpdaterMut<'a, C> {
    fn list<I, II, R>(mut self, items: II, mode: ListElementCreation, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::Nodes<C>),
    {
        let (comp, state, mut r) = self
            .nodes_updater_mut()
            .get_list_updater(mode.use_template());
        let _do_we_have_to_care_about_this_returned_value_ =
            r.render(comp, state, items, |item: I, nodes: NodesUpdater<C>| {
                let mut nodes = HtmlNodesUpdater::new(nodes);
                render(item, crate::Nodes::new(&mut nodes))
            });
        self
    }

    fn list_clone<I, II, R>(self, items: II, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::Nodes<C>),
    {
        self.list(items, ListElementCreation::Clone, render)
    }
}

impl<'a, C: Component> HemsForPartialList<'a, C> for NodesOwned<'a, C> {}
impl<'a, C: Component> HemsForPartialList<'a, C> for StaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'n, C> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'n, C> for StaticNodes<'h, 'n, C> {}
