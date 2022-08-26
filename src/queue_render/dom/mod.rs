use crate::dom::{Element, MaybeAChildNode, ParentAndChild};
use wasm_bindgen::UnwrapThrowExt;

mod list;
mod text;

pub use list::*;
pub use text::*;

pub enum QrNode {
    //ActiveTextNode(Box<dyn QrText>),
    ClonedWsNode(Option<web_sys::Node>),
    ActiveText(QrTextNode),
    List(QrListRepresentative),
}

impl MaybeAChildNode for QrNode {
    fn ws_node(&self) -> Option<&web_sys::Node> {
        match self {
            //Self::ActiveTextNode(a) => a.ws_node(),
            Self::ClonedWsNode(ws) => ws.as_ref(),
            Self::ActiveText(tn) => Some(tn.ws_node()),
            Self::List(r) => r.end_flag_node(),
        }
    }
}

impl QrNode {
    pub fn get_first_element(&self) -> Option<&Element> {
        match self {
            //Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
            Self::ActiveText(_) => None,
            Self::List(_) => None,
        }
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        match self {
            //Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
            Self::ActiveText(_) => None,
            Self::List(_) => None,
        }
    }

    #[cfg(feature = "queue-render")]
    pub fn mark_as_unmounted(&self) {
        match self {
            //Self::ActiveTextNode(tn) => tn.mark_as_unmounted(),
            Self::ClonedWsNode(_) => {}
            Self::ActiveText(tn) => tn.mark_as_unmounted(),
            Self::List(tn) => tn.mark_as_unmounted(),
        }
    }
}
impl Clone for QrNode {
    fn clone(&self) -> Self {
        match self {
            //Self::ActiveTextNode(text) => Self::ClonedWsNode(Some(text.clone_ws_node())),
            Self::ClonedWsNode(wsn) => Self::ClonedWsNode(wsn.as_ref().map(|wsn| {
                wsn.clone_node_with_deep(false)
                    .expect_throw("dom::queue_render::text::Clone for QrNode::clone")
            })),
            Self::ActiveText(tn) => Self::ClonedWsNode(Some(tn.clone_ws_node())),
            Self::List(l) => Self::ClonedWsNode(l.end_flag_node().cloned()),
        }
    }
}
