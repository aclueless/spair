use super::{SvgNodes, SvgNodesOwned, SvgStaticNodes, SvgStaticNodesOwned, SvgTag};
use crate::{
    component::Component,
    render::{
        base::{ElementUpdater, NodesUpdaterMut},
        ListElementCreation,
    },
};

pub trait SemsForPartialList<'a, C: Component>: Sized + NodesUpdaterMut<'a, C> {
    fn list<I, II, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'static str,
        render: R,
    ) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::SvgElement<C>),
    {
        let (comp, state, mut r) = self
            .nodes_updater_mut()
            .get_list_updater(mode.use_template());
        let _do_we_have_to_care_about_this_returned_value_ = r.render(
            comp,
            state,
            items,
            SvgTag(tag),
            |item: I, er: ElementUpdater<C>| render(item, er.into()),
        );
        self
    }

    fn list_clone<I, II, R>(self, items: II, tag: &'static str, render: R) -> Self
    where
        II: Iterator<Item = I>,
        R: Fn(I, crate::SvgElement<C>),
    {
        self.list(items, ListElementCreation::Clone, tag, render)
    }
}

impl<'a, C: Component> SemsForPartialList<'a, C> for SvgNodesOwned<'a, C> {}
impl<'a, C: Component> SemsForPartialList<'a, C> for SvgStaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'n, C> for SvgNodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'n, C> for SvgStaticNodes<'h, 'n, C> {}
