use crate::dom::{AChildNode, Element, MaybeAChildNode};
use wasm_bindgen::UnwrapThrowExt;

mod list;
mod nodes;
mod text;

pub use list::*;
pub use nodes::*;
pub use text::*;

pub enum QrNode {
    ClonedWsNode(Option<web_sys::Node>),
    Text(QrTextNode),
    List(QrListRepresentative),
    Group(QrGroupRepresentative),
}

impl MaybeAChildNode for QrNode {
    fn ws_node(&self) -> Option<&web_sys::Node> {
        match self {
            Self::ClonedWsNode(ws) => ws.as_ref(),
            Self::Text(tn) => Some(tn.ws_node()),
            Self::List(r) => r.end_flag_node(),
            Self::Group(g) => Some(g.end_flag_node()),
        }
    }
}

impl QrNode {
    pub fn get_first_element(&self) -> Option<&Element> {
        match self {
            Self::ClonedWsNode(_) => None,
            Self::Text(_) => None,
            Self::List(_) => None,
            Self::Group(_) => None,
        }
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        match self {
            Self::ClonedWsNode(_) => None,
            Self::Text(_) => None,
            Self::List(_) => None,
            Self::Group(_) => None,
        }
    }

    #[cfg(feature = "queue-render")]
    pub fn mark_as_unmounted(&self) {
        match self {
            Self::ClonedWsNode(_) => {}
            Self::Text(tn) => tn.mark_as_unmounted(),
            Self::List(tn) => tn.mark_as_unmounted(),
            Self::Group(tn) => tn.mark_as_unmounted(),
        }
    }
}
impl Clone for QrNode {
    fn clone(&self) -> Self {
        match self {
            Self::ClonedWsNode(wsn) => Self::ClonedWsNode(wsn.as_ref().map(|wsn| {
                wsn.clone_node_with_deep(false)
                    .expect_throw("dom::queue_render::text::Clone for QrNode::clone")
            })),
            Self::Text(tn) => Self::ClonedWsNode(Some(tn.clone_ws_node())),
            Self::List(l) => Self::ClonedWsNode(l.end_flag_node().cloned()),
            Self::Group(l) => Self::ClonedWsNode(Some(l.end_flag_node().clone())),
        }
    }
}
