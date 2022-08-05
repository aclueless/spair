use crate::component::Component;
use crate::dom::{Key, Keyed, NameSpace};
use crate::render::base::{ElementRender, ElementRenderMut, MakeNodesExtensions, NodesExtensions};
use crate::render::svg::{
    SvgAttributesOnly, SvgElementRender, SvgNameSpace, SvgStaticAttributes, SvgStaticAttributesOnly,
};
use crate::render::ListElementCreation;

pub trait SemsForKeyedList<'a, C: Component>:
    Sized + ElementRenderMut<C> + MakeNodesExtensions<'a>
{
    fn keyed_list_with_render<I, II, G, K, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'a str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        II: IntoIterator<Item = I>,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'u> R: Fn(I, SvgElementRender<'u, C>),
    {
        let fn_render = |item: I, element: ElementRender<C>| {
            fn_render(item, element.into());
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager =
            self.element_render_mut().keyed_list_with_render(
                items,
                mode,
                tag,
                SvgNameSpace::NAMESPACE,
                fn_get_key,
                fn_render,
            );
        self.make_nodes_extensions()
    }

    fn keyed_list<I, II>(self, items: II, mode: ListElementCreation) -> NodesExtensions<'a>
    where
        for<'k> I: Copy + Keyed<'k> + super::SvgListItemRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::key, I::render)
    }

    fn klwr_clone<I, II, G, K, R>(
        self,
        items: II,
        tag: &'a str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        II: IntoIterator<Item = I>,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'u> R: Fn(I, SvgElementRender<'u, C>),
    {
        self.keyed_list_with_render(
            items,
            ListElementCreation::Clone,
            tag,
            fn_get_key,
            fn_render,
        )
    }
}

impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgElementRender<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
