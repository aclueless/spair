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
    fn list<I, II, R>(mut self, items: II, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::Nodes<C>),
    {
        render_list(&mut self, ListElementCreation::New, items, render);
        self
    }

    fn list_clone<I, II, R>(mut self, items: II, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::Nodes<C>),
    {
        render_list(&mut self, ListElementCreation::Clone, items, render);
        self
    }
}

fn render_list<'a, C, T, I, II, R>(updater: &mut T, mode: ListElementCreation, items: II, render: R)
where
    II: Iterator<Item = I>,
    R: Fn(I, crate::Nodes<C>),
    C: Component,
    T: NodesUpdaterMut<'a, C>,
{
    let (comp, state, mut list_updater) = updater
        .nodes_updater_mut()
        .get_list_updater(mode.use_template());
    let _do_we_have_to_care_about_this_returned_value_ =
        list_updater.render(comp, state, items, |item: I, nodes: NodesUpdater<C>| {
            let mut nodes = HtmlNodesUpdater::new(nodes);
            render(item, crate::Nodes::new(&mut nodes))
        });
}

impl<'a, C: Component> HemsForPartialList<'a, C> for NodesOwned<'a, C> {}
impl<'a, C: Component> HemsForPartialList<'a, C> for StaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'n, C> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'n, C> for StaticNodes<'h, 'n, C> {}
