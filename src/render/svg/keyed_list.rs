use crate::{
    component::Component,
    dom::{Key, Keyed},
    render::{
        base::{ElementRender, ElementRenderMut, MakeNodesExtensions, NodesExtensions},
        svg::{
            SvgAttributesOnly, SvgElementRender, SvgStaticAttributes, SvgStaticAttributesOnly,
            SvgTag,
        },
        ListElementCreation,
    },
};

pub trait SemsForKeyedList<'a, C: Component>:
    Sized + ElementRenderMut<C> + MakeNodesExtensions<'a>
{
    fn keyed_list_with_render<I, II, G, K, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'static str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'r> R: Fn(&I, SvgElementRender<'r, C>),
    {
        let fn_render = |item: &I, element: ElementRender<C>| {
            fn_render(item, element.into());
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager = self
            .element_render_mut()
            .keyed_list_with_render(items, mode, SvgTag(tag), fn_get_key, fn_render);
        self.make_nodes_extensions()
    }

    fn keyed_list<I, II>(self, items: II, mode: ListElementCreation) -> NodesExtensions<'a>
    where
        for<'k> I: Keyed<'k> + super::SvgListItemRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::key, I::render)
    }

    fn keyed_lwr_clone<I, II, G, K, R>(
        self,
        items: II,
        tag: &'static str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'r> R: Fn(&I, SvgElementRender<'r, C>),
    {
        self.keyed_list_with_render(
            items,
            ListElementCreation::Clone,
            tag,
            fn_get_key,
            fn_render,
        )
    }

    fn keyed_list_clone<I, II>(self, items: II) -> NodesExtensions<'a>
    where
        for<'k> I: Keyed<'k> + super::SvgListItemRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(
            items,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::key,
            I::render,
        )
    }
}

impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgElementRender<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
