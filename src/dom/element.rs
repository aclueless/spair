use wasm_bindgen::{JsCast, UnwrapThrowExt};

#[derive(Debug)]
pub struct Element {
    pub element_type: super::ElementType,
    pub ws_element: web_sys::Element,
    pub attributes: super::attributes::AttributeList,
    pub nodes: super::nodes::NodeList,
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let ws_element = self
            .ws_element
            .clone_node_with_deep(false)
            .expect_throw("Unable to clone a web_sys::Node");
        let nodes = self.nodes.clone();
        nodes.append_to(&ws_element);

        Self {
            element_type: self.element_type,
            ws_element: ws_element.unchecked_into(),
            nodes,
            // TODO: Should this be cloned?
            attributes: Default::default(),
        }
    }
}

impl Element {
    pub fn new_ns(ns: Option<&'static str>, tag: &str) -> Self {
        let document = crate::utils::document();
        Self {
            element_type: tag.into(),
            ws_element: if ns.is_some() {
                document.create_element_ns(ns, tag)
            } else {
                document.create_element(tag)
            }
            .expect_throw("Unable to create new element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub(crate) fn from_ws_element(ws_element: web_sys::Element) -> Self {
        Self {
            element_type: ws_element.tag_name().to_ascii_lowercase().as_str().into(),
            ws_element,
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.count() == 0
    }

    pub fn ws_element(&self) -> &web_sys::Element {
        &self.ws_element
    }

    // This is intended for use with child component
    pub(crate) fn replace_ws_element(&mut self, ws_element: web_sys::Element) {
        self.ws_element = ws_element;
        self.nodes.append_to(self.ws_element.as_ref());
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(self.ws_element.as_ref())
            .expect_throw("Unable to append child Element to its expected parent");
    }

    pub fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
        parent
            .insert_before(self.ws_element.as_ref(), next_sibling)
            .expect_throw("Unable to insert a child Element to its expected parent");
    }

    pub fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_element.as_ref())
            .expect_throw("Unable to remove a child Element from its parent");
    }
}

enum SelectedOption {
    None,
    Value(String),
    Index(usize),
}

pub struct SelectElementValue(Option<SelectedOption>);

impl SelectElementValue {
    pub fn none() -> Self {
        Self(None)
    }
    pub fn set_selected_value(&mut self, value: Option<&str>) {
        self.0 = Some(
            value
                .map(|value| SelectedOption::Value(value.to_string()))
                .unwrap_or(SelectedOption::None),
        );
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.0 = Some(
            index
                .map(SelectedOption::Index)
                .unwrap_or(SelectedOption::None),
        );
    }

    pub fn set_select_element_value(&self, element: &web_sys::Node) {
        if let Some(selected_option) = self.0.as_ref() {
            let select = element.unchecked_ref::<web_sys::HtmlSelectElement>();
            match selected_option {
                SelectedOption::None => select.set_selected_index(-1),
                SelectedOption::Value(value) => select.set_value(&value),
                SelectedOption::Index(index) => select.set_selected_index(*index as i32),
            }
        }
    }
}

pub struct ElementUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    index: usize,
    status: super::ElementStatus,
    element: &'a mut Element,
}

impl<'a, C> ElementUpdater<'a, C> {
    pub fn into_parts(
        self,
    ) -> (
        &'a crate::component::Comp<C>,
        &'a C,
        super::ElementStatus,
        &'a mut Element,
    ) {
        (self.comp, self.state, self.status, self.element)
    }
}

impl<'a, C: crate::component::Component> ElementUpdater<'a, C> {
    pub(crate) fn new(
        comp: &'a crate::component::Comp<C>,
        state: &'a C,
        element: &'a mut Element,
        status: super::ElementStatus,
    ) -> Self {
        Self {
            comp,
            state,
            index: 0,
            status,
            element,
        }
    }

    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.comp.clone()
    }

    pub fn ws_element(&self) -> &web_sys::Element {
        &self.element.ws_element
    }

    pub fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.element.ws_element.unchecked_ref()
    }

    pub fn element_type(&self) -> super::ElementType {
        self.element.element_type
    }

    pub fn status(&self) -> super::ElementStatus {
        self.status
    }

    pub fn next_index(&mut self) {
        self.index += 1;
    }

    pub fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.element.attributes.store_listener(self.index, listener);
        self.index += 1;
    }

    pub fn check_bool_attribute(&mut self, value: bool) -> bool {
        let rs = self
            .element
            .attributes
            .check_bool_attribute(self.index, value);
        self.index += 1;
        rs
    }

    pub fn check_str_attribute(&mut self, value: &str) -> bool {
        let rs = self
            .element
            .attributes
            .check_str_attribute(self.index, value);
        self.index += 1;
        rs
    }

    pub fn check_i32_attribute(&mut self, value: i32) -> bool {
        let rs = self
            .element
            .attributes
            .check_i32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    pub fn check_u32_attribute(&mut self, value: u32) -> bool {
        let rs = self
            .element
            .attributes
            .check_u32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    pub fn check_f64_attribute(&mut self, value: f64) -> bool {
        let rs = self
            .element
            .attributes
            .check_f64_attribute(self.index, value);
        self.index += 1;
        rs
    }

    pub fn non_keyed_list_updater(
        &mut self,
        mode: super::ListElementCreation,
        tag: &'a str,
    ) -> super::NonKeyedListUpdater<C> {
        super::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut self.element.nodes,
            tag,
            self.element.ws_element.as_ref(),
            None,
            mode.use_template(),
        )
    }

    pub fn list_with_render<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &'a str,
        render: R,
    ) -> super::RememberSettingSelectedOption
    where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.non_keyed_list_updater(mode, tag)
            .html_update(items, render)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
    ) -> super::RememberSettingSelectedOption
    where
        for<'k> I: super::Keyed<'k> + super::ListItem<C>,
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let parent = self.element.ws_element.as_ref();
        let use_template = mode.use_template();

        let mut keyed_list_updater = super::KeyedListUpdater {
            comp: self.comp,
            state: self.state,
            list_context: self.element.nodes.keyed_list_context(
                I::ROOT_ELEMENT_TAG,
                parent,
                items.len(),
                use_template,
            ),
        };
        keyed_list_updater.update(items.into_iter())
    }

    #[cfg(feature = "keyed-list")]
    pub fn update_with_render<I, G, K, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &'a str,
        get_key: G,
        render: R,
    ) -> super::RememberSettingSelectedOption
    where
        I: Copy,
        G: Fn(I) -> K,
        K: Into<super::Key> + PartialEq<super::Key>,
        for<'u> R: super::ListItemRender<'u, I, C>,
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        //let parent = self.element.ws_element.as_ref();
        let use_template = mode.use_template();

        let keyed_list_updater = super::KeyedListUpdater2 {
            comp: self.comp,
            state: self.state,
            list_context: self.element.nodes.keyed_list_context(
                tag,
                self.element.ws_element.as_ref(),
                items.len(),
                use_template,
            ),
            get_key,
            render,
        };
        keyed_list_updater.update_with_render(items.into_iter())
        //crate::dom::RememberSettingSelectedOption
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        // if just created: replace child's root_element with this ws_element
        // first render
        // on the second subsequent render, do nothing.

        if self.status == super::ElementStatus::JustCreated || !child.comp_instance().is_mounted() {
            self.element.ws_element().set_text_content(None);
            child.mount_to(self.element.ws_element());
            self.element
                .nodes
                .store_component_handle(child.comp().into());
        }
    }
}
