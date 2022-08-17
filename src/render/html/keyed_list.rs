use crate::{
    component::Component,
    dom::{Key, Keyed, NameSpace},
    render::{
        base::{ElementRender, ElementRenderMut, MakeNodesExtensions, NodesExtensions},
        html::{
            AttributesOnly, HtmlElementRender, HtmlNameSpace, StaticAttributes,
            StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForKeyedList<'a, C: Component>:
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
        for<'r> R: Fn(I, HtmlElementRender<'r, C>),
    {
        let fn_render = |item: I, element: ElementRender<C>| {
            fn_render(item, element.into());
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager =
            self.element_render_mut().keyed_list_with_render(
                items,
                mode,
                tag,
                HtmlNameSpace::NAMESPACE,
                fn_get_key,
                fn_render,
            );
        self.make_nodes_extensions()
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
        for<'r> R: Fn(I, HtmlElementRender<'r, C>),
    {
        self.keyed_list_with_render(
            items,
            ListElementCreation::Clone,
            tag,
            fn_get_key,
            fn_render,
        )
    }

    fn keyed_list<I, II>(self, items: II, mode: ListElementCreation) -> NodesExtensions<'a>
    where
        for<'k> I: Copy + Keyed<'k> + super::ListItemRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::key, I::render)
    }

    fn kl_clone<I, II>(self, items: II) -> NodesExtensions<'a>
    where
        for<'k> I: Copy + Keyed<'k> + super::ListItemRender<C>,
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

impl<'a, C: Component> HemsForKeyedList<'a, C> for HtmlElementRender<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributesOnly<'a, C> {}
