use wasm_bindgen::UnwrapThrowExt;

mod attributes;
mod element;
mod node;
mod nodes;
mod text;

pub use attributes::*;
pub use element::*;
pub use node::*;
pub use nodes::*;
pub use text::*;

#[cfg(feature = "keyed-list")]
mod keyed_list;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;

#[cfg(feature = "queue-render")]
mod queue_render;
#[cfg(feature = "queue-render")]
pub use queue_render::*;

pub trait NameSpace {
    const NAMESPACE: Option<&'static str>;
}

pub trait ParentAndChild {
    fn ws_node(&self) -> &web_sys::Node;

    fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(self.ws_node())
            .expect_throw("ParentAndChild::append_to");
    }

    fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        parent
            .insert_before(self.ws_node(), next_sibling)
            .expect_throw("ParentAndChild::insert_before");
    }

    fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_node())
            .expect_throw("ParentAndChild::remove_from");
    }
}

impl ParentAndChild for web_sys::Node {
    fn ws_node(&self) -> &web_sys::Node {
        self
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ElementStatus {
    JustCreated,
    Existing,
    JustCloned,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementType {
    Select,
    Input,
    Option,
    TextArea,
    Other,
}

impl From<&str> for ElementType {
    fn from(tag: &str) -> Self {
        match tag {
            "select" => Self::Select,
            "input" => Self::Input,
            "option" => Self::Option,
            "textarea" => Self::TextArea,
            _ => Self::Other,
        }
    }
}
