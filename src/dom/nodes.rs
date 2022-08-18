#[cfg(feature = "keyed-list")]
use super::KeyedList;
#[cfg(feature = "queue-render")]
use super::QrNode;
use super::{
    Element, ElementStatus, NameSpace, Node, OwnedComponent, ParentAndChild, RefComponent, TextNode,
};
use crate::component::{Comp, Component, ComponentHandle};
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
    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.0.drain(..).for_each(|mut node| node.clear(parent));
    }

    /// Just clear the internal Vec of child nodes. The caller must make sure
    /// that the web_sys::Node-child-elements are removed from their parent
    pub fn clear_vec(&mut self) {
        self.0.clear();
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        self.0
            .drain(index..)
            .for_each(|mut node| node.clear(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

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

    pub fn create_new_element_ns(
        &mut self,
        ns: Option<&str>,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let e = Element::new_ns(ns, tag);
        e.insert_before_a_sibling(parent, next_sibling);
        self.0.push(Node::Element(e));
    }

    pub fn check_or_create_element<N: NameSpace>(
        &mut self,
        tag: &str,
        index: usize,
        parent_status: ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> ElementStatus {
        if index == self.0.len() {
            self.create_new_element_ns(N::NAMESPACE, tag, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    pub fn check_or_create_element_for_list<N: NameSpace>(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.create_new_element_ns(N::NAMESPACE, tag, parent, next_sibling);
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

    pub fn store_ref_component(&mut self, index: usize, rc: RefComponent) {
        if index < self.0.len() {
            panic!("Currently, spair expected a ref component to be add to the end of the nodes");
        }
        self.0.push(Node::RefComponent(rc));
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

impl GroupedNodes {
    fn new() -> Self {
        let end_flag_node = crate::utils::document()
            .create_comment("Mark the end of a grouped node list")
            .into();
        Self {
            active_index: None,
            end_flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn set_active_index(&mut self, index: u32, parent: &web_sys::Node) -> ElementStatus {
        if Some(index) != self.active_index {
            self.nodes.clear(parent);
            self.active_index = Some(index);
            ElementStatus::JustCreated
        } else {
            ElementStatus::Existing
        }
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.nodes.clear(parent);
        parent
            .remove_child(&self.end_flag_node)
            .expect_throw("dom::nodes::GroupedNodes::clear remove_child");
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        parent
            .append_child(&self.end_flag_node)
            .expect_throw("dom::nodes::GroupedNodes::append_to append_child");
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn nodes_mut_and_end_flag_node(&mut self) -> (&mut Nodes, &web_sys::Node) {
        (&mut self.nodes, &self.end_flag_node)
    }
}

pub struct AnyComponentHandle(Box<dyn std::any::Any>);

impl<C: Component> From<Comp<C>> for AnyComponentHandle {
    fn from(comp: Comp<C>) -> Self {
        Self(Box::new(ComponentHandle::from(comp)))
    }
}
impl Clone for AnyComponentHandle {
    fn clone(&self) -> Self {
        //
        panic!("Spair does not support mounting a component inside a list item");
    }
}
