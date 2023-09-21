use std::any::TypeId;

#[cfg(feature = "keyed-list")]
use super::KeyedList;
use super::{
    AChildNode, ComponentRef, Element, ElementStatus, ElementTag, InternalTextRender, Node,
    OwnedComponent, RefComponentNode, TextNode,
};
#[cfg(feature = "queue-render")]
use crate::queue_render::dom::QrNode;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct Nodes(Vec<Node>);

pub const FLAG_NAME_FOR_PARTIAL_LIST: &str = "end of a partial list";
pub const FLAG_NAME_FOR_LIST_ENTRY: &str = "start of a list entry";
pub const FLAG_NAME_FOR_MATCH_IF: &str = "end of a match_if";

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
    // use for QrListUpdater, but the list only store elements: Vec<GroupedNodes>.
    // TODO: Should we have an Vec<GroupedNodes> for List and QrList
    // This method is use for QrList, so the items are always GroupedNodes
    pub fn pop_group(&mut self) -> Option<GroupedNodes> {
        match self.0.pop()? {
            Node::GroupedNodes(g) => Some(g),
            _ => None, // Actually, it is a bug if it is not a GroupedNodes
        }
    }

    pub fn insert_group_at(&mut self, index: usize, group: GroupedNodes) {
        self.0.insert(index, Node::GroupedNodes(group));
    }

    pub fn remove_group_at(&mut self, index: usize) -> GroupedNodes {
        match self.0.remove(index) {
            Node::GroupedNodes(g) => g,
            _ => panic!("remove_element_at should be a GroupedNodes"),
        }
    }

    pub fn get_grouped_nodes(&self, index: usize) -> Option<&GroupedNodes> {
        match self.0.get(index) {
            Some(Node::GroupedNodes(group)) => Some(group),
            None => None,
            _ => panic!("dom::nodes::Nodes::get_element expected Node::GroupedNodes"),
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

    pub fn recipe_for_list_entry(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        parent_status: ElementStatus,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> (ElementStatus, &mut GroupedNodes, Option<web_sys::Node>) {
        let item_count = self.0.len();
        let status = if index < item_count {
            parent_status
        } else if !use_template || item_count == 0 {
            self.new_grouped_nodes(FLAG_NAME_FOR_LIST_ENTRY, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            let grouped_nodes = &self.0[0];
            let grouped_nodes = match grouped_nodes {
                Node::GroupedNodes(group) => {
                    let group = group.clone_list_entry();
                    group.insert_before_a_sibling(parent, next_sibling);
                    group
                }
                _ => panic!(
                    "dom::nodes::Nodes::check_or_create_element_for_list expected Node::Element"
                ),
            };
            self.0.push(Node::GroupedNodes(grouped_nodes));
            ElementStatus::JustCloned
        };
        let next_sibling = self
            .get_grouped_nodes(index + 1)
            .map(|g| g.flag_node.clone_node().expect_throw("Clone web_sys::Node"));

        (status, self.get_grouped_nodes_mut(index), next_sibling)
    }

    fn new_grouped_nodes(
        &mut self,
        flag_name: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let gn = GroupedNodes::with_flag_name(flag_name);
        gn.flag_node.insert_before_a_sibling(parent, next_sibling);
        self.0.push(Node::GroupedNodes(gn));
    }

    pub fn grouped_nodes(
        &mut self,
        index: usize,
        flag_name: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> &mut GroupedNodes {
        if index == self.0.len() {
            self.new_grouped_nodes(flag_name, parent, next_sibling);
        }
        self.get_grouped_nodes_mut(index)
    }

    pub fn get_grouped_nodes_mut(&mut self, index: usize) -> &mut GroupedNodes {
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

    pub fn ref_component(
        &mut self,
        index: usize,
        comp_ref: Box<dyn ComponentRef>,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            let rcn = RefComponentNode::new(comp_ref);
            rcn.placeholder_flag()
                .insert_before_a_sibling(parent, next_sibling);
            rcn.mount(parent);
            self.0.push(Node::RefComponent(rcn));
            return;
        }
        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::ref_component2 get_mut")
        {
            Node::RefComponent(rcn) => {
                // if rcn.comp_ref().type_id() != comp_ref.type_id() {
                // always replace the component. If the component is mounted then ChildComp::component_ref()
                // returns None and this will never be reached
                rcn.unmount(parent);
                rcn.replace_comp_ref(comp_ref);
                rcn.mount(parent);
                // }
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

    pub fn update_text(
        &mut self,
        index: usize,
        text: impl InternalTextRender,
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
        value: impl InternalTextRender,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let text = value.to_string();
        let ws_node: web_sys::Node = crate::utils::document().create_text_node(&text).into();
        let text = TextNode::new(value.create_text_node_value(), ws_node);

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

    fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        for n in self.0.iter() {
            n.insert_before_a_sibling(parent, next_sibling);
        }
    }
}

#[derive(Debug)]
pub struct GroupedNodes {
    active_index: Option<TypeId>,
    // `flag_node` marks the boundary of the start/end of this group of nodes
    // It is the start node for a list entry group, but it is the end node for other uses
    flag_node: web_sys::Node,
    nodes: Nodes,
}

impl Clone for GroupedNodes {
    fn clone(&self) -> Self {
        // a GroupedNodes should not be cloned?
        // default: GroupNodes is not cloned, but a GroupedNodes is also used for
        // a list entry, list implementations use GroupedNodes::clone_list_entry
        Self::with_flag(self.clone_flag_node())
    }
}

impl GroupedNodes {
    pub fn with_flag_name(flag_name: &str) -> Self {
        let flag_node = crate::utils::create_comment_node(flag_name);
        Self {
            active_index: None,
            flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn with_flag(flag_node: web_sys::Node) -> Self {
        Self {
            active_index: None,
            flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn flag_node_ref(&self) -> &web_sys::Node {
        &self.flag_node
    }

    fn clone_flag_node(&self) -> web_sys::Node {
        self.flag_node
            .clone_node_with_deep(false)
            .expect_throw("clone GroupedNodes::flag_node")
    }

    pub fn clone_list_entry(&self) -> Self {
        let nodes = self.nodes.clone();
        Self {
            active_index: self.active_index,
            flag_node: self.clone_flag_node(),
            nodes,
        }
    }

    pub fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        self.flag_node.insert_before_a_sibling(parent, next_sibling);
        self.nodes.insert_before_a_sibling(parent, next_sibling);
    }

    pub fn set_active_index(&mut self, index: TypeId, parent: &web_sys::Node) -> ElementStatus {
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
        self.flag_node.remove_from(parent);
    }

    pub fn append_to_parent_with_flag_as_end(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        self.flag_node.append_to(parent);
    }

    pub fn append_to_parent_with_flag_as_start(&self, parent: &web_sys::Node) {
        self.flag_node.append_to(parent);
        self.nodes.append_to(parent);
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Nodes {
        &mut self.nodes
    }

    pub fn nodes_mut_and_flag_node(&mut self) -> (&mut Nodes, &web_sys::Node) {
        (&mut self.nodes, &self.flag_node)
    }

    #[cfg(test)]
    pub fn active_index(&self) -> Option<TypeId> {
        self.active_index
    }
}
