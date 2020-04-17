use wasm_bindgen::{JsCast, UnwrapThrowExt};

#[derive(Debug)]
pub struct Element {
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

    pub fn ws_element(&self) -> &web_sys::Element {
        &self.ws_element
    }

    pub fn create_updater<'a, C>(
        &'a mut self,
        comp: &'a crate::component::Comp<C>,
        status: super::ElementStatus,
    ) -> ElementUpdater<'a, C> {
        let extra = super::Extra {
            comp,
            status,
            index: 0,
        };
        ElementUpdater {
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
            self.create_updater(
                comp,
                if self.is_empty() {
                    super::ElementStatus::JustCreated
                } else {
                    super::ElementStatus::Existing
                },
            ),
        )
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

    pub fn clear(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_element.as_ref())
            .expect_throw("Unable to remove a child Element from its parent");
    }
}

pub struct ElementUpdater<'a, C> {
    pub element: &'a mut Element,
    pub extra: super::Extra<'a, C>,
}

impl<'a, C> ElementUpdater<'a, C> {
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

    pub fn list<I>(mut self, state: &C, items: impl IntoIterator<Item = I>)
    where
        I: crate::renderable::ListItem<C>,
    {
        let parent = self.element.ws_element.as_ref();
        for item in items {
            let element =
                self.element
                    .nodes
                    .item_for_list(I::ROOT_ELEMENT_TAG, &self.extra, parent);
            item.render(state, element);
            self.extra.index += 1;
        }
        self.element.nodes.clear_after(self.extra.index, parent);
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, state: &C, items: impl IntoIterator<Item = I>)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        // How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let parent = self.element.ws_element.as_ref();
        let mut updater =
            self.element
                .nodes
                .keyed_list(I::ROOT_ELEMENT_TAG, parent, &self.extra, items.len());
        updater.update(state, items.into_iter());
    }
}
