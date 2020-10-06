use wasm_bindgen::{JsCast, UnwrapThrowExt};

#[derive(Debug)]
pub struct Element {
    pub element_type: super::ElementType,
    pub ws_element: web_sys::Element,
    pub attributes: super::AttributeList,
    pub nodes: super::NodeList,
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

    pub fn create_context(&mut self, status: super::ElementStatus) -> ElementContext {
        ElementContext {
            index: 0,
            status,
            element: self,
            selected_option: None,
        }
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

pub struct ElementContext<'a> {
    pub(super) index: usize,
    pub(super) status: super::ElementStatus,
    pub(super) element: &'a mut Element,
    pub(super) selected_option: Option<SelectedOption>,
}

pub enum SelectedOption {
    None,
    Value(String),
    Index(i32),
}

pub struct ElementUpdater<'a, C> {
    pub(crate) comp_context: super::CompContext<'a, C>,
    pub(crate) el_context: ElementContext<'a>,
}

impl<'a, C: crate::component::Component> ElementUpdater<'a, C> {
    pub fn state(&self) -> &'a C {
        self.comp_context.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.comp_context.comp.clone()
    }

    pub fn ws_element(&self) -> web_sys::Element {
        self.el_context.element.ws_element.clone()
    }

    pub fn static_attributes(self) -> super::StaticAttributes<'a, C> {
        super::StaticAttributes::new(self)
    }

    pub fn attributes(self) -> super::Attributes<'a, C> {
        super::Attributes::new(self)
    }

    pub fn static_nodes(self) -> super::StaticNodesOwned<'a, C> {
        super::StaticNodesOwned::from_el_updater(self)
    }

    pub fn nodes(self) -> super::NodesOwned<'a, C> {
        super::NodesOwned::from_el_updater(self)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.list_with_render(items, I::ROOT_ELEMENT_TAG, I::render, mode);
    }

    pub fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.el_context.index = 0;

        let parent = self.el_context.element.ws_element.as_ref();
        let use_template = match mode {
            super::ListElementCreation::Clone => true,
            super::ListElementCreation::New => false,
        };
        for item in items {
            let el_context = self
                .el_context
                .element
                .nodes
                .element_context_for_list_element(tag, self.el_context.index, parent, use_template);
            let eu = self.comp_context.element_updater(el_context);
            render(&item, eu);
            self.el_context.index += 1;
        }
        self.el_context
            .element
            .nodes
            .clear_after(self.el_context.index, parent);

        // The hack start in AttributeSetter::value
        self.finish_hacking_for_select_value();
    }

    fn finish_hacking_for_select_value(self) {
        if let Some(selected_option) = self.el_context.selected_option {
            let select = self
                .el_context
                .element
                .ws_element()
                .unchecked_ref::<web_sys::HtmlSelectElement>();
            match selected_option {
                SelectedOption::None => select.set_selected_index(-1),
                SelectedOption::Value(value) => select.set_value(&value),
                SelectedOption::Index(index) => select.set_selected_index(index),
            }
        }
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let parent = self.el_context.element.ws_element.as_ref();
        let use_template = match mode {
            super::ListElementCreation::Clone => true,
            super::ListElementCreation::New => false,
        };

        let mut keyed_list_updater =
            self.comp_context
                .keyed_list_updater(self.el_context.element.nodes.keyed_list_context(
                    I::ROOT_ELEMENT_TAG,
                    parent,
                    items.len(),
                    use_template,
                ));
        keyed_list_updater.update(items.into_iter());

        // The hack start in AttributeSetter::value
        self.finish_hacking_for_select_value();
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        // if just created: replace child's root_element with this ws_element
        // first render
        // on the second subsequent render, do nothing.

        if self.el_context.status == super::ElementStatus::JustCreated
            || !child.comp_instance().is_mounted()
        {
            self.el_context.element.ws_element().set_text_content(None);
            child.mount_to(self.el_context.element.ws_element());
            self.el_context
                .element
                .nodes
                .store_component_handle(child.comp().into());
        }
    }
}
