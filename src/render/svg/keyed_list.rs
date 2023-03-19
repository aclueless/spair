use crate::{
    component::Component,
    dom::ListEntryKey,
    render::{
        base::{ElementUpdaterMut, MakeNodesExtensions, NodesExtensions, NodesUpdater},
        svg::{
            SvgAttributesOnly, SvgElementUpdater, SvgNodes, SvgStaticAttributes,
            SvgStaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait SemsForKeyedList<'a, C: Component>:
    Sized + ElementUpdaterMut<'a, C> + MakeNodesExtensions<'a>
{
    fn keyed_list<I, II, G, K, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, SvgNodes<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let fn_render = |item: I, mut nodes_udpater: NodesUpdater<C>| {
            fn_render(item, SvgNodes::new(&mut nodes_udpater));
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager = self
            .element_updater_mut()
            .keyed_list(items, mode, fn_get_key, fn_render);
        self.make_nodes_extensions()
    }

    fn keyed_list_clone<I, II, G, K, R>(
        self,
        items: II,
        fn_get_key: G,
        fn_render: R,
    ) -> NodesExtensions<'a>
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, SvgNodes<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        self.keyed_list(items, ListElementCreation::Clone, fn_get_key, fn_render)
    }
}

impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
