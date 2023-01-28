#[cfg(feature = "keyed-list")]
use super::KeyedList;
use super::{
    AChildNode, Element, ElementStatus, ElementTag, Node, OwnedComponent, TextNode,
    ComponentRef, RefComponentNode
};
#[cfg(feature = "queue-render")]
use crate::queue_render::dom::QrNode;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct Nodes(Vec<Node>);

impl std::fmt::Debug for Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("[{} nodes]", self.0.len()))
    }
}

impl Nodes {
    #[cfg(test)]
    pub fn nodes_vec(&self) -> &Vec<Node> {
        &self.0
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn remove_from_dom(mut self, parent: &web_sys::Node) {
        self.clear_and_remove_child_from_dom(parent);
    }

    pub fn clear_and_remove_child_from_dom(&mut self, parent: &web_sys::Node) {
        self.0
            .drain(..)
            .for_each(|node| node.remove_from_dom(parent));
    }

    /// Just clear the internal Vec of child nodes. The caller must make sure
    /// that the web_sys::Node-child-elements are removed from their parent
    pub fn clear_vec(&mut self) {
        self.0.clear();
    }

    pub fn remove_from_dom_after(&mut self, index: usize, parent: &web_sys::Node) {
        self.0
            .drain(index..)
            .for_each(|node| node.remove_from_dom(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

    // The following methods, from here until '================' are especically
    // use for QrListUpdater, but the list only store elements: Vec<Element>.
    // TODO: Should we have an Vec<Element> for List and QrList
    // This method is use for QrList, so the items are always Element
    pub fn pop_element(&mut self) -> Option<Element> {
        match self.0.pop()? {
            Node::Element(e) => Some(e),
            _ => None, // Actually, it is a bug if it is not an Element
        }
    }

    pub fn insert_element_at(&mut self, index: usize, element: Element) {
        self.0.insert(index, Node::Element(element));
    }

    pub fn remove_element_at(&mut self, index: usize) -> Element {
        match self.0.remove(index) {
            Node::Element(e) => e,
            _ => panic!("remove_element_at should be an Element"),
        }
    }

    pub fn get_element(&self, index: usize) -> Option<&Element> {
        match self.0.get(index) {
            Some(Node::Element(element)) => Some(element),
            None => None,
            _ => panic!("dom::nodes::Nodes::get_element expected Node::Element"),
        }
    }

    // '================'
    // end of QrListUpdater only methods

    pub fn get_element_mut(&mut self, index: usize) -> &mut Element {
        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::get_element_mut")
        {
            Node::Element(element) => element,
            _ => panic!("dom::nodes::Nodes::get_element_mut expected Node::Element"),
        }
    }

    pub fn create_new_element_ns<E: ElementTag>(
        &mut self,
        tag: E,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let e = Element::new_ns(tag);
        e.insert_before_a_sibling(parent, next_sibling);
        self.0.push(Node::Element(e));
    }

    pub fn check_or_create_element<E: ElementTag>(
        &mut self,
        tag: E,
        index: usize,
        parent_status: ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> ElementStatus {
        if index == self.0.len() {
            self.create_new_element_ns(tag, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    pub fn check_or_create_element_for_list<E: ElementTag>(
        &mut self,
        tag: E,
        index: usize,
        parent: &web_sys::Node,
        parent_status: ElementStatus,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            parent_status
        } else if !use_template || item_count == 0 {
            self.create_new_element_ns(tag, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            let element = self.0[0].clone();
            match &element {
                Node::Element(element) => element.insert_before_a_sibling(parent, next_sibling),
                _ => panic!(
                    "dom::nodes::Nodes::check_or_create_element_for_list expected Node::Element"
                ),
            }
            self.0.push(element);
            ElementStatus::JustCloned
        }
    }

    pub fn grouped_nodes(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> &mut GroupedNodes {
        if index == self.0.len() {
            let gn = GroupedNodes::new();
            gn.end_flag_node
                .insert_before_a_sibling(parent, next_sibling);
            //.expect_throw("dom::nodes::Nodes::grouped_nodes insert_before");
            self.0.push(Node::GroupedNodes(gn));
        }

        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::grouped_nodes get_mut")
        {
            Node::GroupedNodes(gn) => gn,
            _ => panic!("dom::nodes::Nodes::grouped_nodes expected Node::GroupedNodes"),
        }
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list(&mut self) -> &mut KeyedList {
        // KeyedList is the only item of self.0, because spair only supports
        // whole-keyed-list (keyed-list managed the whole content of the parent element)

        // Partial keyed list is currently not supported yet. If partial-keyed-list
        // support is adding to spair, this method will be changed to take an `index` argument.

        if self.0.is_empty() {
            self.0.push(Node::KeyedList(Default::default()));
        }

        match self
            .0
            .first_mut()
            .expect_throw("dom::nodes::Nodes::keyed_list first_mut")
        {
            Node::KeyedList(list) => list,
            _ => panic!("dom::nodes::Nodes::keyed_list expected Node::KeyedList"),
        }
    }

    pub fn ref_component(&mut self, index: usize, comp_ref: Box<dyn ComponentRef>, parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,) {
        if index == self.0.len() {
            let rcn = RefComponentNode::new(comp_ref);
            rcn.placeholder_flag().insert_before_a_sibling(parent, next_sibling);
            rcn.mount(parent);
            self.0.push(Node::RefComponent(rcn));
            return;
        }
        match self.0.get_mut(index)
            .expect_throw("dom::nodes::Nodes::ref_component2 get_mut") {
            Node::RefComponent(rcn) => if rcn.comp_ref().type_id() != comp_ref.type_id() {
                rcn.replace_comp_ref(comp_ref);
                rcn.mount(parent);
            }
            _ => panic!("dom::nodes::Nodes::ref_component2 expected Node::RefComponent2"),
        }
    }

    fn get_owned_component_mut(&mut self, index: usize) -> &mut OwnedComponent {
        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::get_owned_component_mut get_mut")
        {
            Node::OwnedComponent(oc) => oc,
            _ => panic!("dom::nodes::Nodes::get_owned_component_mut expected Node::OwnedComponent"),
        }
    }

    pub fn owned_component(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        ccc: impl FnOnce() -> OwnedComponent,
    ) -> &mut OwnedComponent {
        if self.0.len() <= index {
            let oc = ccc();
            oc.insert_before_a_sibling(parent, next_sibling);
            let oc = Node::OwnedComponent(oc);
            self.0.push(oc);
            self.get_owned_component_mut(index)
        } else {
            let oc = self.get_owned_component_mut(index);
            if oc.is_empty() {
                *oc = ccc();
                oc.insert_before_a_sibling(parent, next_sibling);
            }
            oc
        }
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        self.0.first().and_then(|n| n.get_first_element())
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        self.0.last().and_then(|n| n.get_last_element())
    }

    pub fn static_text(
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

    pub fn update_text(
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
                .expect_throw("dom::nodes::Nodes::update_text get_mut")
            {
                Node::Text(text_node) => text_node.update_text(text),
                _ => panic!("dom::nodes::Nodes::update_text expected Node::Text"),
            }
        }
    }

    fn add_text_node(
        &mut self,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let text = TextNode::new(text);
        text.insert_before_a_sibling(parent, next_sibling);
        self.0.push(Node::Text(text));
    }

    #[cfg(feature = "queue-render")]
    pub fn add_qr_node(&mut self, qr: QrNode) {
        self.0.push(Node::QrNode(qr));
    }

    #[cfg(feature = "queue-render")]
    pub fn get_qr_node(&mut self, index: usize) -> &mut QrNode {
        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::get_qr_node get_mut")
        {
            Node::QrNode(qr) => qr,
            _ => panic!("dom::nodes::Nodes::get_qr_node expected Node::QrNode"),
        }
    }
}

pub struct GroupedNodes {
    active_index: Option<u32>,
    // `end_flag_node` marks the boundary of the end of this group of nodes
    end_flag_node: web_sys::Node,
    nodes: Nodes,
}

impl Clone for GroupedNodes {
    fn clone(&self) -> Self {
        // a GroupedNodes should not be cloned?
        Self::new()
    }
}

impl Default for GroupedNodes {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupedNodes {
    pub fn new() -> Self {
        let end_flag_node = crate::utils::create_comment_node("Mark the end of a grouped node list");
        Self {
            active_index: None,
            end_flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn with_flag(end_flag_node: web_sys::Node) -> Self {
        Self {
            active_index: None,
            end_flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn end_flag_node(&self) -> &web_sys::Node {
        &self.end_flag_node
    }

    pub fn set_active_index(&mut self, index: u32, parent: &web_sys::Node) -> ElementStatus {
        if Some(index) != self.active_index {
            self.nodes.clear_and_remove_child_from_dom(parent);
            self.active_index = Some(index);
            ElementStatus::JustCreated
        } else {
            ElementStatus::Existing
        }
    }

    pub fn remove_from_dom(self, parent: &web_sys::Node) {
        self.nodes.remove_from_dom(parent);
        self.end_flag_node.remove_from(parent);
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        self.end_flag_node.append_to(parent);
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn nodes_mut_and_end_flag_node(&mut self) -> (&mut Nodes, &web_sys::Node) {
        (&mut self.nodes, &self.end_flag_node)
    }
}

