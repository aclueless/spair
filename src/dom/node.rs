use std::any::Any;

use crate::component::{ChildComp, Component, ComponentHandle};

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
    RefComponent(RefComponent),
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
            Self::RefComponent(_) => "Node::RefComponent",
            Self::OwnedComponent(_) => "Node::OwnedComponent",
            #[cfg(feature = "queue-render")]
            Self::QrNode(_) => "Node::QrNode",
        };
        f.write_fmt(format_args!("[{}]", name))
    }
}

pub struct RefComponent {
    _comp: Box<dyn std::any::Any>,
    root_node: web_sys::Node,
}

impl RefComponent {
    pub fn new<C: Component>(comp: &ChildComp<C>) -> Self {
        let v = comp.comp_instance();
        let root_node: &web_sys::Node = v.root_element().ws_element().ws_node();
        let handle = ComponentHandle::from(comp.comp());
        Self {
            _comp: Box::new(handle),
            root_node: root_node.clone(),
        }
    }
}

impl Clone for RefComponent {
    fn clone(&self) -> Self {
        panic!("Spair does not support using component_ref inside a list item");
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
}

impl Node {
    pub fn remove_from_dom(&mut self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => {
                element.mark_as_unmounted();
                element.remove_from(parent);
            }
            Self::Text(text) => text.remove_from(parent),
            Self::GroupedNodes(g) => g.clear(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.clear(parent),
            Self::RefComponent(rc) => {
                rc.root_node.remove_from(parent);
            }
            Self::OwnedComponent(oc) => {
                if let Some(wsn) = oc.root_node.as_ref() {
                    wsn.remove_from(parent);
                }
            }
            #[cfg(feature = "queue-render")]
            Self::QrNode(qr) => {
                qr.mark_as_unmounted();
                qr.remove_from(parent);
            }
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.append_to(parent),
            Self::Text(text) => text.append_to(parent),
            Self::GroupedNodes(g) => g.append_to(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.append_to(parent),
            // This is actually never reachable?
            Self::RefComponent(rc) => rc.root_node.append_to(parent),
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
}
