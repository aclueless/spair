use super::SvgListItemRender;
use super::{SvgNameSpace, SvgNodes, SvgNodesOwned, SvgStaticNodes, SvgStaticNodesOwned};
use crate::component::Component;
use crate::render::base::{ElementRender, NodesRenderMut};
use crate::render::ListElementCreation;

pub trait SemsForPartialList<'a, C: Component>: Sized + NodesRenderMut<C> {
    fn list_with_render<I, II, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'a str,
        render: R,
    ) where
        I: Copy,
        II: IntoIterator<Item = I>,
        for<'r> R: Fn(I, crate::Svg<'r, C>),
    {
        let mut r = self
            .nodes_render_mut()
            .get_list_render(tag, mode.use_template());
        let _do_we_have_to_care_about_this_returned_value_ = r
            .render::<SvgNameSpace, _, _, _>(items, |item: I, er: ElementRender<C>| {
                render(item, er.into())
            });
    }

    fn lwr_clone<I, II, R>(self, items: II, tag: &'a str, render: R)
    where
        I: Copy,
        II: IntoIterator<Item = I>,
        for<'r> R: Fn(I, crate::Svg<'r, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn list<I, II>(self, items: II, mode: ListElementCreation)
    where
        I: Copy + SvgListItemRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I, II>(self, items: II)
    where
        I: Copy + SvgListItemRender<C>,
        II: IntoIterator<Item = I>,
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
