use super::SvgListItemRender;
use super::{SvgNameSpace, SvgNodes, SvgNodesOwned, SvgStaticNodes, SvgStaticNodesOwned};
use crate::component::Component;
use crate::dom::NameSpace;
use crate::render::base::{ElementRender, NodesRenderMut};
use crate::render::ListElementCreation;

pub trait SemsForPartialList<'a, C: Component>: Sized + NodesRenderMut<C> {
    fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: ListElementCreation,
        tag: &'a str,
        render: R,
    ) where
        I: Copy,
        for<'u> R: Fn(I, crate::Svg<'u, C>),
    {
        let mut r = self.nodes_render_mut().get_list_render(
            tag,
            SvgNameSpace::NAMESPACE,
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
        for<'u> R: Fn(I, crate::Svg<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn list<I>(self, items: impl IntoIterator<Item = I>, mode: ListElementCreation)
    where
        I: Copy,
        I: SvgListItemRender<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I>(self, items: impl IntoIterator<Item = I>)
    where
        I: Copy,
        I: SvgListItemRender<C>,
    {
        self.list_with_render(
            items,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::render,
        )
    }
}

impl<'a, C: Component> SemsForPartialList<'a, C> for SvgNodesOwned<'a, C> {}
impl<'a, C: Component> SemsForPartialList<'a, C> for SvgStaticNodesOwned<'a, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'h, C> for SvgNodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> SemsForPartialList<'h, C> for SvgStaticNodes<'h, 'n, C> {}
