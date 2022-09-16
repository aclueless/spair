use super::ListRender;
use crate::{
    component::{Comp, Component},
    dom::{AttributeValueList, Element, ElementStatus},
    render::ListElementCreation,
};
use wasm_bindgen::UnwrapThrowExt;

#[cfg(feature = "keyed-list")]
use crate::{
    dom::{ElementTag, Key},
    render::base::{
        KeyedListContext, KeyedListRender, RememberSettingSelectedOption, RenderContext,
    },
};

pub trait ElementUpdaterMut<C: Component> {
    fn element_render(&self) -> &ElementUpdater<C>;
    fn element_render_mut(&mut self) -> &mut ElementUpdater<C>;
}

impl<C, T> ElementUpdaterMut<C> for &mut T
where
    C: Component,
    T: ElementUpdaterMut<C>,
{
    fn element_render(&self) -> &ElementUpdater<C> {
        (**self).element_render()
    }
    fn element_render_mut(&mut self) -> &mut ElementUpdater<C> {
        (**self).element_render_mut()
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

    fn is_update_mode(&self) -> bool {
        self.update_mode
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

    pub fn must_render_attribute<T>(
        &mut self,
        value: T,
        check: impl FnOnce(&mut AttributeValueList, usize, T) -> bool,
    ) -> bool {
        if self.is_static_mode() {
            self.status == ElementStatus::JustCreated
        } else {
            let rs = check(self.element.attribute_list_mut(), self.index, value);
            self.index += 1;
            rs
        }
    }

    pub fn set_bool_attribute(&mut self, name: &str, value: bool) {
        if !self.must_render_attribute(value, AttributeValueList::check_bool_attribute) {
            return;
        }
        self.element.ws_element().set_bool_attribute(name, value);
    }

    pub fn set_str_attribute(&mut self, name: &str, value: &str) {
        if !self.must_render_attribute(value, AttributeValueList::check_str_attribute) {
            return;
        }
        self.element.ws_element().set_str_attribute(name, value);
    }

    pub fn set_string_attribute(&mut self, name: &str, value: String) {
        if !self.must_render_attribute(value.as_str(), AttributeValueList::check_str_attribute) {
            return;
        }
        self.element.ws_element().set_str_attribute(name, &value);
    }

    pub fn set_i32_attribute(&mut self, name: &str, value: i32) {
        if !self.must_render_attribute(value, AttributeValueList::check_i32_attribute) {
            return;
        }
        self.element.ws_element().set_attribute(name, value);
    }

    pub fn set_u32_attribute(&mut self, name: &str, value: u32) {
        if !self.must_render_attribute(value, AttributeValueList::check_u32_attribute) {
            return;
        }
        self.element.ws_element().set_attribute(name, value);
    }

    pub fn set_f64_attribute(&mut self, name: &str, value: f64) {
        if !self.must_render_attribute(value, AttributeValueList::check_f64_attribute) {
            return;
        }
        self.element.ws_element().set_attribute(name, value);
    }

    /// Always checked.
    pub fn checked(&self, value: bool) {
        self.element.ws_element().checked(value);
    }

    pub fn enabled(&self, value: bool) {
        self.element.ws_element().enabled(value);
    }

    pub fn class(&mut self, class_name: &str) {
        let (changed, old_value) = if self.is_update_mode() {
            let rs = self
                .element
                .attribute_list_mut()
                .check_str_attribute_and_return_old_value(self.index, class_name);
            self.index += 1;
            rs
        } else {
            (self.status == ElementStatus::JustCreated, None)
        };
        if let Some(old_value) = old_value {
            self.element.ws_element().remove_class(&old_value);
        }
        if changed {
            self.element.ws_element().add_class(class_name);
        }
    }

    /// Make sure that value of `class_name` does not change between calls.
    pub fn class_if(&mut self, class_on: bool, class_name: &str) {
        if !self.must_render_attribute(class_on, AttributeValueList::check_bool_attribute) {
            return;
        }
        if class_on {
            self.element.ws_element().add_class(class_name);
        } else {
            self.element.ws_element().remove_class(class_name);
        }
    }

    pub fn class_or(&mut self, first: bool, first_class: &str, second_class: &str) {
        if !self.must_render_attribute(first, AttributeValueList::check_bool_attribute) {
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
        if !self.must_render_attribute(value, AttributeValueList::check_bool_attribute) {
            return;
        }
        if value {
            self.element
                .ws_html_element()
                .focus()
                .expect_throw("render::base::element::ElementRender::focus");
        }
    }

    pub fn href(&mut self, route: &C::Routes) {
        // Should `route` be stored in attribute list as an PartialEq object?
        // Is that possible? It may avoid calling `route.url()` if the route does not change.
        use crate::routing::Routes;
        let url = route.url();
        if !self.must_render_attribute(url.as_str(), AttributeValueList::check_str_attribute) {
            return;
        }
        self.element.ws_element().set_str_attribute("href", &url);
    }

    pub fn id(&mut self, id: &str) {
        if !self.must_render_attribute(id, AttributeValueList::check_str_attribute) {
            return;
        }
        self.element.ws_element().set_id(id);
    }

    pub fn list_render(&mut self, mode: ListElementCreation) -> (&Comp<C>, &C, ListRender) {
        let (parent, nodes) = self.element.ws_node_and_nodes_mut();
        let lr = ListRender::new(nodes, parent, self.status, None, mode.use_template());
        (self.comp, self.state, lr)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list_with_render<E, I, II, G, K, R>(
        &mut self,
        items: II,
        mode: ListElementCreation,
        tag: E,
        fn_get_key: G,
        fn_render: R,
    ) -> RememberSettingSelectedOption
    where
        E: ElementTag,
        II: IntoIterator<Item = I>,
        G: Fn(&I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementUpdater<'er, C>),
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let use_template = mode.use_template();
        let (parent, nodes) = self.element.ws_node_and_nodes_mut();
        let mut keyed_list_render = KeyedListRender::new(
            KeyedListContext::new(nodes.keyed_list(), tag, items.len(), parent, use_template),
            RenderContext::new(self.comp, self.state, fn_get_key, fn_render),
        );
        keyed_list_render.update(items.into_iter())
    }
}
