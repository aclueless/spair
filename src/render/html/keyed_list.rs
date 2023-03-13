use crate::{
    component::Component,
    dom::ListEntryKey,
    render::{
        base::{ElementUpdaterMut, MakeNodesExtensions, NodesExtensions, NodesUpdater},
        html::{
            AttributesOnly, HtmlElementUpdater, HtmlNodesUpdater, Nodes, StaticAttributes,
            StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForKeyedList<'a, C: Component>:
    Sized + ElementUpdaterMut<'a, C> + MakeNodesExtensions<'a>
{
    fn keyed_list<I, II, G, K, R>(
        mut self,
        entries: II,
        mode: ListElementCreation,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, Nodes<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let fn_render = |entry: I, nodes_updater: NodesUpdater<C>| {
            let mut nodes = HtmlNodesUpdater::new(nodes_updater);
            fn_render(entry, Nodes::new(&mut nodes));
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager = self
            .element_updater_mut()
            .keyed_list(entries, mode, fn_get_key, fn_render);
        self.make_nodes_extensions()
    }

    fn keyed_list_clone<I, II, G, K, R>(
        self,
        entries: II,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, Nodes<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        self.keyed_list(entries, ListElementCreation::Clone, fn_get_key, fn_render)
    }
}

impl<'a, C: Component> HemsForKeyedList<'a, C> for HtmlElementUpdater<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForKeyedList<'a, C> for StaticAttributesOnly<'a, C> {}
