use super::{ListItemRender, Nodes, NodesOwned, StaticNodes, StaticNodesOwned};
use crate::{
    component::Component,
    render::{
        base::{ElementRender, NodesRenderMut},
        html::HtmlTag,
        ListElementCreation,
    },
};

pub trait HemsForPartialList<'a, C: Component>: Sized + NodesRenderMut<C> {
    fn list_with_render<I, II, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'static str,
        render: R,
    ) where
        II: Iterator<Item = I>,
        for<'r> R: Fn(&I, crate::Element<'r, C>),
    {
        let tag = HtmlTag(tag);

        let (comp, state, mut r) = self.nodes_render_mut().get_list_render(mode.use_template());
        let _do_we_have_to_care_about_this_returned_value_ =
            r.render(comp, state, items, tag, |item: &I, er: ElementRender<C>| {
                render(item, er.into())
            });
    }

    fn lwr_clone<I, II, R>(self, items: II, tag: &'static str, render: R)
    where
        II: Iterator<Item = I>,
        for<'r> R: Fn(&I, crate::Element<'r, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn list<I, II>(self, items: II, mode: ListElementCreation)
    where
        I: ListItemRender<C>,
        II: Iterator<Item = I>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I, II>(self, items: II)
    where
        I: ListItemRender<C>,
        II: Iterator<Item = I>,
    {
        self.list_with_render(
            items,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::render,
        )
    }
}

impl<'a, C: Component> HemsForPartialList<'a, C> for NodesOwned<'a, C> {}
impl<'a, C: Component> HemsForPartialList<'a, C> for StaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'h, C> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> HemsForPartialList<'h, C> for StaticNodes<'h, 'n, C> {}
