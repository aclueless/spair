use super::ListUpdater;
use crate::{
    component::{Comp, Component},
    dom::{AttributeValueAsString, AttributeValueList, Element, ElementStatus},
    render::ListElementCreation,
};
use wasm_bindgen::UnwrapThrowExt;

#[cfg(feature = "keyed-list")]
use crate::{
    dom::ListEntryKey,
    render::base::{
        KeyedListContext, KeyedListUpdater, KeyedListUpdaterContext, NodesUpdater,
        RememberSettingSelectedOption,
    },
};

pub trait ElementUpdaterMut<'updater, C: Component> {
    fn element_updater(&self) -> &ElementUpdater<C>;
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<'updater, C>;
}

impl<'updater, C, T> ElementUpdaterMut<'updater, C> for &mut T
where
    C: Component,
    T: ElementUpdaterMut<'updater, C>,
{
    fn element_updater(&self) -> &ElementUpdater<C> {
        (**self).element_updater()
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<'updater, C> {
        (**self).element_updater_mut()
    }
}

pub struct ElementUpdater<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    update_mode: bool,
    index: usize,
    status: ElementStatus,
    element: &'a mut Element,
}

impl<'a, C: Component> ElementUpdater<'a, C> {
    pub fn new(
        comp: &'a crate::component::Comp<C>,
        state: &'a C,
        element: &'a mut Element,
        status: ElementStatus,
    ) -> Self {
        Self {
            comp,
            state,
            update_mode: true,
            index: 0,
            status,
            element,
        }
    }

    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> Comp<C> {
        self.comp.clone()
    }

    pub fn into_parts(self) -> (&'a Comp<C>, &'a C, ElementStatus, &'a mut Element) {
        (self.comp, self.state, self.status, self.element)
    }

    pub fn element(&self) -> &Element {
        self.element
    }

    pub fn element_mut(&mut self) -> &mut Element {
        self.element
    }

    #[cfg(feature = "queue-render")]
    pub(crate) fn status(&self) -> ElementStatus {
        self.status
    }

    // pub(crate) fn index(&self) -> usize {
    //     self.index
    // }

    // pub(crate) fn next_index(&mut self) {
    //     self.index += 1;
    // }

    pub fn set_static_mode(&mut self) {
        self.update_mode = false;
    }

    pub fn set_update_mode(&mut self) {
        self.update_mode = true;
    }

    fn is_static_mode(&self) -> bool {
        !self.update_mode
    }

    pub fn require_set_listener(&mut self) -> bool {
        if self.is_static_mode() {
            if self.status == ElementStatus::Existing {
                // self.store_listener will not be invoked.
                // We must update the index here to count over the static event.
                self.index += 1;
                false
            } else {
                // A cloned element requires its event handlers to be set because the event
                // listeners are not cloned.
                true
            }
        } else {
            true
        }
    }

    pub fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        let index = self.index;
        self.element_mut()
            .attribute_list_mut()
            .store_listener(index, listener);
        self.index += 1;
    }

    pub fn attribute(&mut self, name: &str, value: impl AttributeValueAsString) {
        if self.is_static_mode() {
            if self.status == ElementStatus::JustCreated {
                // never have to detect changes, so just
                // only render the attribute value on ws_element
                value.set(name, self.element.ws_element());
            }
        } else {
            self.element.attribute(self.index, name, value);
            self.index += 1;
        }
    }

    pub fn bool_value_change(&mut self, new_value: bool) -> bool {
        if self.is_static_mode() {
            self.status == ElementStatus::JustCreated
        } else {
            let rs = self
                .element
                .attribute_list_mut()
                .bool_value_change(self.index, new_value);
            self.index += 1;
            rs
        }
    }

    pub fn i32_value_change(&mut self, new_value: i32) -> bool {
        if self.is_static_mode() {
            self.status == ElementStatus::JustCreated
        } else {
            let rs = self
                .element
                .attribute_list_mut()
                .i32_value_change(self.index, new_value);
            self.index += 1;
            rs
        }
    }

    pub fn value_change<T>(
        &mut self,
        new_value: T,
        check: impl FnOnce(&mut AttributeValueList, usize, T) -> (bool, Option<String>),
    ) -> (bool, Option<String>) {
        if self.is_static_mode() {
            (self.status == ElementStatus::JustCreated, None)
        } else {
            let rs = check(self.element.attribute_list_mut(), self.index, new_value);
            self.index += 1;
            rs
        }
    }

    pub fn str_value_change(&mut self, new_value: &str) -> (bool, Option<String>) {
        self.value_change(Some(new_value), AttributeValueList::option_str_value_change)
    }

    pub fn option_str_value_change(&mut self, new_value: Option<&str>) -> (bool, Option<String>) {
        self.value_change(new_value, AttributeValueList::option_str_value_change)
    }

    /// Always checked.
    pub fn checked(&self, value: bool) {
        self.element.ws_element().checked(value);
    }

    pub fn enabled(&self, value: bool) {
        self.element.ws_element().enabled(value);
    }

    pub fn class(&mut self, class_name: &str) {
        let (changed, old_value) = self.str_value_change(class_name);
        if let Some(old_value) = old_value {
            self.element.ws_element().remove_class(&old_value);
        }
        if changed {
            self.element.ws_element().add_class(class_name);
        }
    }

    /// Make sure that value of `class_name` does not change between calls.
    pub fn class_if(&mut self, class_on: bool, class_name: &str) {
        if !self.bool_value_change(class_on) {
            return;
        }
        if class_on {
            self.element.ws_element().add_class(class_name);
        } else {
            self.element.ws_element().remove_class(class_name);
        }
    }

    pub fn class_or(&mut self, first: bool, first_class: &str, second_class: &str) {
        if !self.bool_value_change(first) {
            return;
        }
        if first {
            self.element.ws_element().add_class(first_class);
            self.element.ws_element().remove_class(second_class);
        } else {
            self.element.ws_element().remove_class(first_class);
            self.element.ws_element().add_class(second_class);
        }
    }

    pub fn focus(&mut self, value: bool) {
        if !self.bool_value_change(value) {
            return;
        }
        if value {
            self.element
                .ws_html_element()
                .focus()
                .expect_throw("render::base::element::ElementUpdater::focus");
        }
    }

    pub fn href(&mut self, route: &C::Routes) {
        // Should `route` be stored in attribute list as an PartialEq object?
        // Is that possible? It may avoid calling `route.url()` if the route does not change.
        use crate::routing::Routes;
        let url = route.url();
        self.attribute("href", url);
    }

    pub fn id(&mut self, id: &str) {
        if self
            .element
            .attribute_list_mut()
            .option_str_value_change(self.index, Some(id))
            .0
        {
            self.element.ws_element().set_id(id);
        }
    }

    pub fn list_updater(&mut self, mode: ListElementCreation) -> (&Comp<C>, &C, ListUpdater) {
        let (parent, nodes) = self.element.ws_node_and_nodes_mut();
        let lr = ListUpdater::new(nodes, parent, self.status, None, mode.use_template());
        (self.comp, self.state, lr)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I, II, G, K, R>(
        &mut self,
        items: II,
        mode: ListElementCreation,
        fn_get_key: G,
        fn_render: R,
    ) -> RememberSettingSelectedOption
    where
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        for<'updater> R: Fn(I, NodesUpdater<'updater, C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let use_template = mode.use_template();
        let (parent, nodes) = self.element.ws_node_and_nodes_mut();
        let mut keyed_list_updater = KeyedListUpdater::new(
            KeyedListContext::new(nodes.keyed_list(), items.len(), parent, use_template),
            KeyedListUpdaterContext::new(self.comp, self.state, fn_get_key, fn_render),
        );
        keyed_list_updater.update(items.into_iter())
    }
}
