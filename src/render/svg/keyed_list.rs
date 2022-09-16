use crate::{
    component::Component,
    dom::{Key, Keyed},
    render::{
        base::{ElementUpdater, ElementUpdaterMut, MakeNodesExtensions, NodesExtensions},
        svg::{
            SvgAttributesOnly, SvgElementUpdater, SvgStaticAttributes, SvgStaticAttributesOnly,
            SvgTag,
        },
        ListElementCreation,
    },
};

pub trait SemsForKeyedList<'a, C: Component>:
    Sized + ElementUpdaterMut<C> + MakeNodesExtensions<'a>
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
        for<'r> R: Fn(I, SvgElementUpdater<'r, C>),
    {
        let fn_render = |item: I, element: ElementUpdater<C>| {
            fn_render(item, element.into());
        };
        let _select_element_value_will_be_set_on_dropping_of_the_manager = self
            .element_updater_mut()
            .keyed_list_with_render(items, mode, SvgTag(tag), fn_get_key, fn_render);
        self.make_nodes_extensions()
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
        for<'r> R: Fn(I, SvgElementUpdater<'r, C>),
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
        for<'k> I: Keyed<'k> + super::SvgElementRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(items, mode, I::ELEMENT_TAG, I::key, I::render)
    }

    fn keyed_list_clone<I, II>(self, items: II) -> NodesExtensions<'a>
    where
        for<'k> I: Keyed<'k> + super::SvgElementRender<C>,
        II: IntoIterator<Item = I>,
    {
        self.keyed_list_with_render(
            items,
            ListElementCreation::Clone,
            I::ELEMENT_TAG,
            I::key,
            I::render,
        )
    }
}

impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForKeyedList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
