use super::{SvgNodes, SvgNodesOwned, SvgStaticNodes, SvgStaticNodesOwned};
use crate::{
    component::Component,
    render::{
        base::{NodesUpdater, NodesUpdaterMut},
        ListElementCreation,
    },
};

pub trait SemsForPartialList<'a, C: Component>: Sized + NodesUpdaterMut<'a, C> {
    fn list<I, II, R>(mut self, items: II, mode: ListElementCreation, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, SvgNodes<C>),
    {
        let (comp, state, mut r) = self
            .nodes_updater_mut()
            .get_list_updater(mode.use_template());
        let _do_we_have_to_care_about_this_returned_value_ =
            r.render(comp, state, items, |item: I, mut nodes: NodesUpdater<C>| {
                render(item, SvgNodes::new(&mut nodes))
            });
        self
    }

    fn list_clone<I, II, R>(self, items: II, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, SvgNodes<C>),
    {
        self.list(items, ListElementCreation::Clone, render)
    }
}

impl<'a, C: Component> SemsForPartialList<'a, C> for SvgNodesOwned<'a, C> {}
impl<'a, C: Component> SemsForPartialList<'a, C> for SvgStaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'n, C> for SvgNodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'n, C> for SvgStaticNodes<'h, 'n, C> {}
