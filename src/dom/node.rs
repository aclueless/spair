use super::{AnyComponentHandle, Element, GroupedNodeList, ParentAndChild, TextNode};

#[derive(Clone)]
pub enum Node {
    Element(Element),
    Text(TextNode),
    GroupedNodeList(GroupedNodeList),
    // #[cfg(feature = "keyed-list")]
    // KeyedList(super::KeyedList),
    ComponentHandle(AnyComponentHandle),
    // #[cfg(feature = "queue-render")]
    // QueueRendering(QueueRendering),
}

impl Node {
    pub fn clear(&mut self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.remove_from(parent),
            Self::Text(text) => text.remove_from(parent),
            Self::GroupedNodeList(g) => g.clear(parent),
            // #[cfg(feature = "keyed-list")]
            // Self::KeyedList(list) => list.clear(parent),
            Self::ComponentHandle(_) => {
                // The component is the only child of an element
                parent.set_text_content(None);
                // The NodeList will drop the ComponentHandle in its.clear()
            }
            // #[cfg(feature = "queue-render")]
            // Self::QueueRendering(qr) => qr.remove_from(parent),
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.append_to(parent),
            Self::Text(text) => text.append_to(parent),
            Self::GroupedNodeList(g) => g.append_to(parent),
            // #[cfg(feature = "keyed-list")]
            // Self::KeyedList(list) => list.append_to(parent),
            Self::ComponentHandle(_) => {
                // TODO: Not sure what to do here???
                unreachable!("Node::ComponentHandle::append_to() is unreachable???");
            }
            // #[cfg(feature = "queue-render")]
            // Self::QueueRendering(qr) => qr.append_to(parent),
        }
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        match self {
            Self::Element(element) => Some(element),
            Self::Text(_) => None,
            Self::GroupedNodeList(g) => g.nodes().get_first_element(),
            // #[cfg(feature = "keyed-list")]
            // Self::KeyedList(list) => list.get_first_element(),
            Self::ComponentHandle(_) => None,
            // #[cfg(feature = "queue-render")]
            // Self::QueueRendering(qr) => qr.get_first_element(),
        }
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        match self {
            Self::Element(element) => Some(element),
            Self::Text(_) => None,
            Self::GroupedNodeList(g) => g.nodes().get_last_element(),
            // #[cfg(feature = "keyed-list")]
            // Self::KeyedList(list) => list.get_last_element(),
            Self::ComponentHandle(_) => None,
            // #[cfg(feature = "queue-render")]
            // Self::QueueRendering(qr) => qr.get_last_element(),
        }
    }
}
