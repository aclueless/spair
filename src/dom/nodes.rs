use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct NodeList(pub Vec<Node>);

impl std::fmt::Debug for NodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("NodeList of {} items", self.0.len()))
    }
}

impl NodeList {
    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn clear_raw(&mut self) {
        self.0.clear();
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.0.drain(..).for_each(|mut node| node.clear(parent));
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        self.0
            .drain(index..)
            .for_each(|mut node| node.clear(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

    pub fn get_element(&mut self, index: usize) -> &mut super::Element {
        match self
            .0
            .get_mut(index)
            .expect_throw("Expect an element node at the given index")
        {
            Node::Element(element) => element,
            _ => panic!("Why not an element?"),
        }
    }

    fn create_new_element(
        &mut self,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        self.0.push(Node::Element(super::Element::new_in(
            tag,
            parent,
            next_sibling,
        )));
    }

    pub(super) fn check_or_create_element(
        &mut self,
        tag: &str,
        index: usize,
        parent_status: super::ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> super::ElementStatus {
        if index == self.0.len() {
            self.create_new_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    pub fn check_or_create_element_for_non_keyed_list(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> super::ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            super::ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.create_new_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            let element = self.0[0].clone();
            match &element {
                Node::Element(element) => element.insert_before(parent, next_sibling),
                _ => panic!("non-keyed-list: internal bug?"),
            }
            self.0.push(element);
            super::ElementStatus::JustCloned
        }
    }

    fn fragmented_node_list(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> &mut FragmentedNodeList {
        if index == self.0.len() {
            let fnl = FragmentedNodeList::default();
            parent
                .insert_before(fnl.end_node.as_ref(), next_sibling)
                .expect_throw("Unable to insert fragmented node into its parent node");
            self.0.push(Node::FragmentedNodeList(fnl));
        }

        match self
            .0
            .get_mut(index)
            .expect_throw("Expect a fragmented node list at the given index")
        {
            Node::FragmentedNodeList(fragmented_node_list) => fragmented_node_list,
            _ => panic!("Why not a fragmented_node_list?"),
        }
    }

    pub(super) fn static_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        }
    }

    pub(super) fn update_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        } else {
            match self
                .0
                .get_mut(index)
                .expect_throw("Expect a text node at the given index")
            {
                Node::Text(text_node) => text_node.update_text(text),
                _ => panic!("Why not a text node?"),
            }
        }
    }

    fn add_text_node(
        &mut self,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let text = super::Text::new(text);
        text.insert_before(parent, next_sibling);
        self.0.push(Node::Text(text));
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list_context<'a>(
        &'a mut self,
        root_item_tag: &str,
        parent: &'a web_sys::Node,
        exact_count_of_new_items: usize,
        use_template: bool,
    ) -> super::KeyedListContext<'a> {
        if self.0.is_empty() {
            self.0.push(Node::KeyedList(super::KeyedList::default()));
        }

        match self
            .0
            .first_mut()
            .expect_throw("Expect a keyed list as the first item of the node list")
        {
            Node::KeyedList(list) => list.create_context(
                root_item_tag,
                exact_count_of_new_items,
                parent,
                use_template,
            ),
            _ => panic!("Why not a keyed list?"),
        }
    }

    pub fn store_component_handle(&mut self, any: AnyComponentHandle) {
        let any = Node::ComponentHandle(any);
        if let Some(first) = self.0.first_mut() {
            *first = any;
        } else {
            self.0.push(any);
        }
    }
}

#[derive(Clone)]
pub enum Node {
    Element(super::Element),
    Text(super::Text),
    FragmentedNodeList(FragmentedNodeList),
    #[cfg(feature = "keyed-list")]
    KeyedList(super::KeyedList),
    ComponentHandle(AnyComponentHandle),
}

impl Node {
    fn clear(&mut self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.remove_from(parent),
            Self::Text(text) => text.remove_from(parent),
            Self::FragmentedNodeList(mi) => mi.clear(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.clear(parent),
            Self::ComponentHandle(_) => {
                // The component is the only child of an element
                parent.set_text_content(None);
                // And the NodeList drop the ComponentHandle in its.clear()
            }
        }
    }

    fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.append_to(parent),
            Self::Text(text) => text.append_to(parent),
            Self::FragmentedNodeList(mi) => mi.append_to(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.append_to(parent),
            Self::ComponentHandle(_) => {
                // TODO: Not sure what to do here???
                unreachable!("Node::ComponentHandle::append_to() is unreachable???");
            }
        }
    }
}

pub struct AnyComponentHandle(Box<dyn std::any::Any>);

impl Clone for AnyComponentHandle {
    fn clone(&self) -> Self {
        //
        panic!("Spair does not support mounting a component inside a list item");
    }
}

impl<C: crate::component::Component> From<crate::component::Comp<C>> for AnyComponentHandle {
    fn from(ch: crate::component::Comp<C>) -> Self {
        Self(Box::new(crate::component::ComponentHandle::from(ch)))
    }
}

// Manage a match-if arm or a partial-non-keyed-list
pub struct FragmentedNodeList {
    active_index: Option<usize>,
    // `end_node` marks the boundary of this fragment
    end_node: web_sys::Node,
    nodes: NodeList,
}

impl Clone for FragmentedNodeList {
    fn clone(&self) -> Self {
        // a FragmentedNodeList should not be cloned?
        Default::default()
    }
}

impl Default for FragmentedNodeList {
    fn default() -> Self {
        let end_node = crate::utils::document()
            .create_comment("Mark the end of a fragmented node list")
            .into();
        Self {
            active_index: None,
            end_node,
            nodes: NodeList::default(),
        }
    }
}

impl FragmentedNodeList {
    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.nodes.clear(parent);
        parent
            .remove_child(&self.end_node)
            .expect_throw("Unable to remove FragmentedNodeList.end_node from its parent");
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        parent.append_child(&self.end_node).expect_throw(
            "Unable to append a FragmentedNodeList's end node to its expected parent",
        );
    }
}

pub struct MatchIfUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    match_if: &'a mut FragmentedNodeList,
}

impl<'a, C: crate::component::Component> MatchIfUpdater<'a, C> {
    pub fn render_on_arm_index(mut self, index: usize) -> super::NodesOwned<'a, C> {
        let status = if Some(index) != self.match_if.active_index {
            self.match_if.nodes.clear(self.parent.as_ref());
            self.match_if.active_index = Some(index);
            super::ElementStatus::JustCreated
        } else {
            super::ElementStatus::Existing
        };

        super::NodesOwned::new(NodeListUpdater {
            comp: self.comp,
            state: self.state,

            index: 0,
            parent_status: status,
            nodes: &mut self.match_if.nodes,
            parent: self.parent,
            next_sibling: Some(&self.match_if.end_node),
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: super::SelectElementValue::none(),
        })
    }
}

pub(crate) struct NodeListUpdater<'a, C> {
    pub(in crate::dom) comp: &'a crate::component::Comp<C>,
    pub(in crate::dom) state: &'a C,

    pub(in crate::dom) index: usize,
    pub(in crate::dom) parent_status: super::ElementStatus,
    pub(in crate::dom) parent: &'a web_sys::Node,
    pub(in crate::dom) next_sibling: Option<&'a web_sys::Node>,
    pub(in crate::dom) nodes: &'a mut NodeList,
    #[cfg(feature = "partial-non-keyed-list")]
    pub(in crate::dom) select_element_value: super::SelectElementValue,
}

#[cfg(feature = "partial-non-keyed-list")]
impl<'a, C> Drop for NodeListUpdater<'a, C> {
    fn drop(&mut self) {
        self.select_element_value
            .set_select_element_value(self.parent);
    }
}

impl<'a, C: crate::component::Component> NodeListUpdater<'a, C> {
    pub fn from_el_updater(eu: super::ElementUpdater<'a, C>) -> Self {
        Self {
            comp: eu.comp,
            state: eu.state,
            index: 0,
            parent_status: eu.status,
            parent: eu.element.ws_element.as_ref(),
            next_sibling: None,
            nodes: &mut eu.element.nodes,
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: eu.select_element_value,
        }
    }

    pub(super) fn get_element_and_increase_index(&mut self, tag: &str) -> super::HtmlUpdater<C> {
        let status = self.nodes.check_or_create_element(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element(self.index);
        self.index += 1;
        super::ElementUpdater::new(self.comp, self.state, element, status).into()
    }

    #[cfg(feature = "svg")]
    pub(super) fn get_svg_element_and_increase_index(&mut self) -> super::SvgUpdater<C> {
        let status = self.nodes.check_or_create_svg_element_ns(
            "svg",
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element(self.index);
        self.index += 1;
        super::SvgUpdater::new(self.comp, self.state, element, status)
    }

    pub(super) fn get_match_if_updater(&mut self) -> MatchIfUpdater<C> {
        let match_if = self
            .nodes
            .fragmented_node_list(self.index, self.parent, self.next_sibling);
        self.index += 1;
        MatchIfUpdater {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            match_if,
        }
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        let use_template = mode.use_template();
        let fragmented_node_list =
            self.nodes
                .fragmented_node_list(self.index, self.parent, self.next_sibling);
        self.index += 1;
        let mut non_keyed_list_updater = super::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut fragmented_node_list.nodes,
            tag,
            self.parent,
            Some(&fragmented_node_list.end_node),
            use_template,
        );
        let _select_element_value_will_be_set_on_dropping =
            non_keyed_list_updater.update(items, render);
    }
}

pub trait DomBuilder<C: crate::component::Component> {
    fn require_render(&self) -> bool;
    fn just_created(&self) -> bool;
    fn next_index(&mut self);
    fn get_element_and_increase_index(&mut self, tag: &str) -> super::HtmlUpdater<C>;
    fn get_match_if_and_increase_index(&mut self) -> MatchIfUpdater<C>;
    fn store_raw_wrapper(&mut self, element: crate::dom::Element);
    #[cfg(feature = "svg")]
    fn get_svg_element_and_increase_index(&mut self) -> crate::dom::SvgUpdater<C>;
}
