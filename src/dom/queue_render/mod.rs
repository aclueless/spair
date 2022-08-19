use super::{Element, ParentAndChild};
use wasm_bindgen::UnwrapThrowExt;

mod text;
mod attribute;

pub use text::*;
pub use attribute::*;

pub enum QrNode {
    ActiveTextNode(Box<dyn QrText>),
    ClonedWsNode(Option<web_sys::Node>),
}

impl ParentAndChild for QrNode {
    fn ws_node(&self) -> &web_sys::Node {
        match self {
            Self::ActiveTextNode(a) => a.ws_node(),
            Self::ClonedWsNode(ws) => ws
                .as_ref()
                .expect_throw("dom::queue_render::text::ParentAndChild for QrNode::ws_node"),
        }
    }
}

impl QrNode {
    pub fn get_first_element(&self) -> Option<&Element> {
        match self {
            Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
        }
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        match self {
            Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
        }
    }
}
impl Clone for QrNode {
    fn clone(&self) -> Self {
        match self {
            Self::ActiveTextNode(text) => Self::ClonedWsNode(Some(text.clone_ws_node())),
            Self::ClonedWsNode(wsn) => Self::ClonedWsNode(wsn.as_ref().map(|wsn| {
                wsn.clone_node_with_deep(false)
                    .expect_throw("dom::queue_render::text::Clone for QrNode::clone")
            })),
        }
    }
}
