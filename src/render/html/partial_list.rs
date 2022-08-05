use super::HtmlNameSpace;
use super::ListItemRender;
use super::{Nodes, NodesOwned, StaticNodes, StaticNodesOwned};
use crate::component::Component;
use crate::dom::NameSpace;
use crate::render::base::{ElementRender, NodesRenderMut};
use crate::render::ListElementCreation;

pub trait HemsForPartialList<'a, C: Component>: Sized + NodesRenderMut<C> {
    fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: ListElementCreation,
        tag: &'a str,
        render: R,
    ) where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        let mut r = self.nodes_render_mut().get_list_render(
            tag,
            HtmlNameSpace::NAMESPACE,
            mode.use_template(),
        );
        let _do_we_have_to_care_about_this_returned_value_ = r
            .render(items, |item: I, er: ElementRender<C>| {
                render(item, er.into())
            });
    }

    fn lwr_clone<I, R>(self, items: impl IntoIterator<Item = I>, tag: &'a str, render: R)
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn list<I>(self, items: impl IntoIterator<Item = I>, mode: ListElementCreation)
    where
        I: Copy,
        I: ListItemRender<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I>(self, items: impl IntoIterator<Item = I>)
    where
        I: Copy,
        I: ListItemRender<C>,
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
