use crate::{
    component::Component,
    dom::ListItemKey,
    render::{
        base::{ElementUpdater, ElementUpdaterMut, MakeNodesExtensions, NodesExtensions},
        html::{
            AttributesOnly, HtmlElementUpdater, HtmlTag, StaticAttributes, StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForKeyedList<'a, C: Component>:
    Sized + ElementUpdaterMut<'a, C> + MakeNodesExtensions<'a>
{
    fn keyed_list<I, II, G, K, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'static str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, HtmlElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        let fn_render = |item: I, element: ElementUpdater<C>| {
            fn_render(item, element.into());
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager = self
            .element_updater_mut()
            .keyed_list(items, mode, HtmlTag(tag), fn_get_key, fn_render);
        self.make_nodes_extensions()
    }

    fn keyed_list_clone<I, II, G, K, R>(
        self,
        items: II,
        tag: &'static str,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, HtmlElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        self.keyed_list(
            items,
            ListElementCreation::Clone,
            tag,
            fn_get_key,
            fn_render,
        )
    }
}

impl<'a, C: Component> HemsForKeyedList<'a, C> for HtmlElementUpdater<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributesOnly<'a, C> {}
