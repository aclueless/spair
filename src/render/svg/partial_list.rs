use super::SvgListItemRender;
use super::{SvgListRender, SvgNodes, SvgNodesOwned, SvgStaticNodes, SvgStaticNodesOwned};
use crate::component::Component;
use crate::render::base::NodesRenderMut;
use crate::render::ListElementCreation;

// TODO: Is it possible to merge this and SvgListRender into ListRender using generic?

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
        let r = self
            .nodes_render_mut()
            .get_list_render(tag, mode.use_template());
        let mut r = SvgListRender::new(r);
        let _do_we_have_to_care_about_this_returned_value_ = r.render_list(items, render);
    }

    fn lwr_clone<I, R>(self, items: impl IntoIterator<Item = I>, tag: &'a str, render: R)
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Svg<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn lwr_new<I, R>(self, items: impl IntoIterator<Item = I>, tag: &'a str, render: R)
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Svg<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::New, tag, render)
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
