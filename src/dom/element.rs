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
    pub fn new(tag: &str) -> Self {
        Self {
            element_type: tag.into(),
            ws_element: crate::utils::document()
                .create_element(tag)
                .expect_throw("Unable to create new element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub fn new_in(tag: &str, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) -> Self {
        let element = super::Element::new(tag);
        element.insert_before(parent, next_sibling);
        element
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
    select_element_value: SelectElementValue,
}

impl<'a, C> ElementUpdater<'a, C> {
    pub fn into_parts(
        self,
    ) -> (
        &'a crate::component::Comp<C>,
        &'a C,
        super::ElementStatus,
        &'a mut Element,
        SelectElementValue,
    ) {
        (
            self.comp,
            self.state,
            self.status,
            self.element,
            self.select_element_value,
        )
    }

    pub fn select_element_value_mut(&mut self) -> &mut SelectElementValue {
        &mut self.select_element_value
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
            select_element_value: SelectElementValue::none(),
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

    pub fn clear(self) {
        let parent = self.element.ws_element.as_ref();
        self.element.nodes.clear(parent);
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

    pub fn set_selected_value(&mut self, value: Option<&str>) {
        self.select_element_value.set_selected_value(value);
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.select_element_value.set_selected_index(index);
    }

    pub fn static_attributes(self) -> super::StaticAttributes<'a, C> {
        super::StaticAttributes::from(self)
    }

    pub fn nodes(self) -> super::NodesOwned<'a, C> {
        super::NodesOwned::from(self)
    }

    pub fn static_nodes(self) -> super::StaticNodesOwned<'a, C> {
        super::StaticNodesOwned::from(self)
    }

    pub fn render(self, value: impl crate::renderable::Render<C>) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    pub fn render_ref(
        self,
        value: &impl crate::renderable::RenderRef<C>,
    ) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    pub fn r#static(
        self,
        value: impl crate::renderable::StaticRender<C>,
    ) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let static_nodes = nodes_owned.static_nodes_ref();
        value.render(static_nodes);
        nodes_owned
    }

    pub fn update_text(self, text: &str) -> super::NodesOwned<'a, C> {
        let nodes_owned = self.nodes();
        nodes_owned.update_text(text)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        let parent = self.element.ws_element.as_ref();
        let use_template = mode.use_template();

        let mut non_keyed_list_updater = super::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut self.element.nodes,
            tag,
            parent,
            None,
            use_template,
        );
        let _must_set_select_element_value = non_keyed_list_updater.html_update(items, render);

        // The hack start in AttributeSetter::value
        self.select_element_value.set_select_element_value(parent);
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
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
        keyed_list_updater.update(items.into_iter());

        // The hack start in AttributeSetter::value
        self.select_element_value.set_select_element_value(parent);
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

#[cfg(feature = "svg")]
impl<'a, C: crate::component::Component> ElementUpdater<'a, C> {
    pub fn svg_nodes(self) -> super::SvgNodesOwned<'a, C> {
        super::SvgNodesOwned::from(self)
    }

    pub fn svg_static_nodes(self) -> super::SvgStaticNodesOwned<'a, C> {
        super::SvgStaticNodesOwned::from(self)
    }

    pub fn svg_render(self, value: impl super::SvgRender<C>) -> super::SvgNodesOwned<'a, C> {
        let mut nodes_owned = self.svg_nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    // pub fn render_ref(
    //     self,
    //     value: &impl crate::renderable::RenderRef<C>,
    // ) -> super::NodesOwned<'a, C> {
    //     let mut nodes_owned = self.nodes();
    //     let nodes = nodes_owned.nodes_ref();
    //     value.render(nodes);
    //     nodes_owned
    // }

    pub fn svg_static(self, value: impl super::SvgStaticRender<C>) -> super::SvgNodesOwned<'a, C> {
        let mut nodes_owned = self.svg_nodes();
        let static_nodes = nodes_owned.static_nodes_ref();
        value.render(static_nodes);
        nodes_owned
    }

    pub fn svg_list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, super::SvgUpdater<'c, C>),
    {
        let parent = self.element.ws_element.as_ref();
        let use_template = mode.use_template();

        let mut non_keyed_list_updater = super::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut self.element.nodes,
            tag,
            parent,
            None,
            use_template,
        );
        non_keyed_list_updater.svg_update(items, render);
    }

    pub fn svg(self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> super::NodesOwned<'a, C> {
        let nodes = self.nodes();
        nodes.svg(f)
    }
}
