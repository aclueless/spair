use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub struct Element {
    pub(super) ws_element: web_sys::Element,
    pub(super) attributes: super::AttributeList,
    pub(super) nodes: super::NodeList,
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
            ws_element: ws_element.unchecked_into(),
            nodes,
            // Should this be cloned?
            attributes: Default::default(),
        }
    }
}

impl Element {
    pub(super) fn new(tag: &str) -> Self {
        Self {
            ws_element: crate::utils::document()
                .create_element(tag)
                .expect_throw("Unable to create new element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub(crate) fn from_ws_element(ws_element: web_sys::Element) -> Self {
        Self {
            ws_element,
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.count() == 0
    }

    pub(super) fn create_handle<'a, C>(
        &'a mut self,
        comp: &'a crate::component::Comp<C>,
        status: super::ElementStatus,
    ) -> ElementHandle<'a, C> {
        let extra = super::Extra {
            comp,
            status,
            index: 0,
        };
        ElementHandle {
            element: self,
            extra,
        }
    }

    pub(crate) fn create_context<'a, C>(
        &'a mut self,
        comp: &'a crate::component::Comp<C>,
    ) -> crate::component::Context<'a, C> {
        crate::component::Context::new(
            comp,
            self.create_handle(
                comp,
                if self.is_empty() {
                    super::ElementStatus::JustCreated
                } else {
                    super::ElementStatus::Existing
                },
            ),
        )
    }

    pub(super) fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(self.ws_element.as_ref())
            .expect_throw("Unable to append child Element to its expected parent");
    }

    pub(super) fn insert_before(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        parent
            .insert_before(self.ws_element.as_ref(), next_sibling)
            .expect_throw("Unable to insert a child Element to its expected parent");
    }

    pub(super) fn clear(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_element.as_ref())
            .expect_throw("Unable to remove a child Element from its parent");
    }
}

pub struct ElementHandle<'a, C> {
    pub(super) element: &'a mut Element,
    pub(super) extra: super::Extra<'a, C>,
}

impl<'a, C> ElementHandle<'a, C> {
    pub fn comp(&self) -> crate::component::Comp<C> {
        self.extra.comp.clone()
    }

    pub fn static_attributes(self) -> super::StaticAttributes<'a, C> {
        super::StaticAttributes::new(self)
    }

    pub fn attributes(self) -> super::Attributes<'a, C> {
        super::Attributes::new(self)
    }

    pub fn static_nodes(self) -> super::StaticNodes<'a, C> {
        super::StaticNodes::from_handle(self)
    }

    pub fn nodes(self) -> super::Nodes<'a, C> {
        super::Nodes::from_handle(self)
    }

    // This must be here (not on dom::nodes::Nodes) to prevent adding items to the parent element before the list.
    pub fn list<I>(mut self, items: impl IntoIterator<Item = I>, state: &C)
    where
        I: crate::renderable::ListItem<C>,
    {
        let parent = self.element.ws_element.as_ref();
        for item in items {
            let element =
                self.element
                    .nodes
                    .item_for_list(I::ROOT_ELEMENT_TAG, &self.extra, parent);
            item.render(element, state);
            self.extra.index += 1;
        }
        self.element.nodes.clear_after(self.extra.index, parent);
    }
}
