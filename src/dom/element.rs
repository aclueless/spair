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

    // This is intended for use with child component
    pub(crate) fn replace_ws_element(&mut self, ws_element: web_sys::Element) {
        self.ws_element = ws_element;
        self.nodes.append_to(self.ws_element.as_ref());
    }

    pub fn create_updater<'a, C: crate::component::Component>(
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
            select_value: None,
        }
    }

    pub(crate) fn create_context<'a, C: crate::component::Component>(
        &'a mut self,
        comp: &'a crate::component::Comp<C>,
        child_components: &'a C::Components,
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
            child_components,
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

    pub fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_element.as_ref())
            .expect_throw("Unable to remove a child Element from its parent");
    }
}

pub struct ElementUpdater<'a, C: crate::component::Component> {
    pub element: &'a mut Element,
    pub extra: super::Extra<'a, C>,
    pub select_value: Option<String>,
}

impl<'a, C: crate::component::Component> ElementUpdater<'a, C> {
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
        // Reset the index, because it may used by attributes
        // TODO: Avoid this to eliminate the possibility of a bug appear in the future because of this?
        //  * Maybe a solution is similar to keyed_list: Create its own updater
        self.extra.index = 0;

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

        // The hack start in AttributeSetter::value
        self.finish_hacking_for_select_value();
    }

    fn finish_hacking_for_select_value(self) {
        if let Some(value) = self.select_value {
            self.element
                .ws_element()
                .unchecked_ref::<web_sys::HtmlSelectElement>()
                .set_value(&value);
        }
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, state: &C, items: impl IntoIterator<Item = I>)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        // TODO: How to avoid this? The current implementation requires knowing the exact number of items,
        // we need to collect items into a vec to know exact size
        let items: Vec<_> = items.into_iter().collect();

        let parent = self.element.ws_element.as_ref();
        let mut updater =
            self.element
                .nodes
                .keyed_list(I::ROOT_ELEMENT_TAG, parent, &self.extra, items.len());
        updater.update(state, items.into_iter());

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

        if self.extra.status == super::ElementStatus::JustCreated
            || !child.comp_instance().is_mounted()
        {
            child.mount_to(self.element.ws_element());
            self.element
                .nodes
                .store_component_handle(child.comp().into());
        }
    }
}
