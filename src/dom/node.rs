use std::any::Any;

use crate::component::{Component, ComponentHandle};

#[cfg(feature = "keyed-list")]
use super::KeyedList;
#[cfg(feature = "queue-render")]
use super::MaybeAChildNode;
use super::{AChildNode, Element, ElementStatus, GroupedNodes, TextNode};
#[cfg(feature = "queue-render")]
use crate::queue_render::dom::QrNode;

#[derive(Clone)]
pub enum Node {
    Element(Element),
    Text(TextNode),
    GroupedNodes(GroupedNodes),
    #[cfg(feature = "keyed-list")]
    KeyedList(KeyedList),
    RefComponent(RefComponentNode),
    OwnedComponent(OwnedComponent),
    #[cfg(feature = "queue-render")]
    QrNode(QrNode),
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let name = match self {
            Self::Element(_) => "Node::Element",
            Self::Text(_) => "Node::Text",
            Self::GroupedNodes(_) => "Node::GroupedNodes",
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(_) => "Node::KeyedList",
            // This is actually never reachable?
            Self::RefComponent(_) => "Node::RefComponent2",
            Self::OwnedComponent(_) => "Node::OwnedComponent",
            #[cfg(feature = "queue-render")]
            Self::QrNode(_) => "Node::QrNode",
        };
        f.write_fmt(format_args!("[{name}]"))
    }
}

pub struct CompRef<C: Component> {
    pub comp: ComponentHandle<C>,
    pub ws_node: web_sys::Node,
}

pub trait ComponentRef {
    fn type_id(&self) -> std::any::TypeId;
    fn root_node(&self) -> &web_sys::Node;
}

impl<C: Component> ComponentRef for CompRef<C> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<C>()
    }

    fn root_node(&self) -> &web_sys::Node {
        &self.ws_node
    }
}

pub struct RefComponentNode {
    comp_ref: Box<dyn ComponentRef>,
    placeholder_flag: web_sys::Node,
}

impl RefComponentNode {
    pub fn new(comp_ref: Box<dyn ComponentRef>) -> Self {
        Self {
            comp_ref,
            placeholder_flag: crate::utils::create_comment_node(
                "A flag to mark a child component place",
            ),
        }
    }

    pub fn comp_ref(&self) -> &dyn ComponentRef {
        self.comp_ref.as_ref()
    }

    pub fn replace_comp_ref(&mut self, comp_ref: Box<dyn ComponentRef>) {
        self.comp_ref = comp_ref;
    }

    pub fn placeholder_flag(&self) -> &web_sys::Node {
        &self.placeholder_flag
    }

    // better name?
    pub fn mount(&self, parent: &web_sys::Node) {
        self.comp_ref
            .root_node()
            .insert_before_a_sibling(parent, Some(&self.placeholder_flag));
    }

    // better name?
    pub fn unmount(&self, parent: &web_sys::Node) {
        self.comp_ref.root_node().remove_from(parent);
    }

    pub fn remove_all_from(&self, parent: &web_sys::Node) {
        self.comp_ref.root_node().remove_from(parent);
        self.placeholder_flag.remove_from(parent);
    }

    pub fn append_all_to(&self, parent: &web_sys::Node) {
        self.comp_ref.root_node().append_to(parent);
        self.placeholder_flag.append_to(parent);
    }

    pub fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        self.comp_ref
            .root_node()
            .insert_before_a_sibling(parent, next_sibling);
        self.placeholder_flag
            .insert_before_a_sibling(parent, next_sibling);
    }
}

impl Clone for RefComponentNode {
    fn clone(&self) -> Self {
        const MSG: &str = "Spair does not support using component_ref inside a list item";
        log::error!("{MSG}");
        panic!("{MSG}");
    }
}

pub struct OwnedComponent {
    // A status value of 'JustCreated' means the element is just created.
    // At creation, the root node is created, but the component is not rendered yet.
    status: ElementStatus,
    comp: Option<Box<dyn Any>>,
    root_node: Option<web_sys::Node>,
}

impl Clone for OwnedComponent {
    fn clone(&self) -> Self {
        Self {
            status: ElementStatus::JustCloned,
            comp: None,
            root_node: None,
        }
    }
}

impl OwnedComponent {
    pub fn new(comp: Box<dyn Any>, root_node: web_sys::Node) -> Self {
        Self {
            status: ElementStatus::JustCreated,
            comp: Some(comp),
            root_node: Some(root_node),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.comp.is_none()
    }

    pub fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if let Some(node) = self.root_node.as_ref() {
            node.insert_before_a_sibling(parent, next_sibling);
        }
    }

    pub fn get_any_component_mut(&mut self) -> Option<&mut Box<dyn Any>> {
        self.comp.as_mut()
    }

    pub fn just_created(&self) -> bool {
        self.status == ElementStatus::JustCreated
    }

    pub(crate) fn set_status_to_existing(&mut self) {
        self.status = ElementStatus::Existing;
    }
}

impl Node {
    pub fn remove_from_dom(self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => {
                // Just remove the web_sys::Node from the parent web_sys::Node.
                // The children will be drop together with the element. We don't
                // need to remove them from their parent wes_sys::Node
                element.remove_from(parent);
            }
            Self::Text(text) => text.remove_from(parent),
            // This will be stopped when reaching an actual Node::Element
            Self::GroupedNodes(g) => g.remove_from_dom(parent),
            // This will be stopped when reaching an actual Node::Element
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.remove_from_dom(parent),
            Self::RefComponent(rc) => {
                rc.remove_all_from(parent);
            }
            Self::OwnedComponent(oc) => {
                if let Some(wsn) = oc.root_node.as_ref() {
                    wsn.remove_from(parent);
                }
            }
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => qr.remove_from(parent),
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.append_to(parent),
            Self::Text(text) => text.append_to(parent),
            Self::GroupedNodes(g) => g.append_to_parent_with_flag_as_end(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.append_to(parent),
            // This is actually never reachable?
            Self::RefComponent(rc) => rc.append_all_to(parent),
            Self::OwnedComponent(oc) => {
                if let Some(wsn) = oc.root_node.as_ref() {
                    wsn.append_to(parent);
                }
            }
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => qr.append_to(parent),
        }
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        match self {
            Self::Element(element) => Some(element),
            Self::Text(_) => None,
            Self::GroupedNodes(g) => g.nodes().get_first_element(),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.get_first_element(),
            // Should this return the RefComponent::root_node (wrapped in dom::Element)?
            Self::RefComponent(_) => None,
            Self::OwnedComponent(_) => None,
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => qr.get_first_element(),
        }
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        match self {
            Self::Element(element) => Some(element),
            Self::Text(_) => None,
            Self::GroupedNodes(g) => g.nodes().get_last_element(),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.get_last_element(),
            // Should this return the RefComponent::root_node (wrapped in dom::Element)?
            Self::RefComponent(_) => None,
            Self::OwnedComponent(_) => None,
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => qr.get_last_element(),
        }
    }

    pub fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        match self {
            Self::Element(element) => element.insert_before_a_sibling(parent, next_sibling),
            Self::Text(text) => text.insert_before_a_sibling(parent, next_sibling),
            Self::GroupedNodes(g) => g.insert_before_a_sibling(parent, next_sibling),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.insert_before_a_sibling(parent, next_sibling),
            // This is actually never reachable?
            Self::RefComponent(rc) => rc.insert_before_a_sibling(parent, next_sibling),
            Self::OwnedComponent(oc) => {
                if let Some(wsn) = oc.root_node.as_ref() {
                    wsn.insert_before_a_sibling(parent, next_sibling);
                }
            }
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => qr.insert_before_a_sibling(parent, next_sibling),
        }
    }
}
