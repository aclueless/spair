use wasm_bindgen::UnwrapThrowExt;

pub mod attribute_types;
mod attributes;
mod element;
#[cfg(feature = "keyed-list")]
mod keyed_list;
mod nodes;
mod non_keyed_list;

use attribute_types::AsStr;

// All items from these modules are visible here but `dom` is private,
// the `lib.rs` must selectively export items from `dom` to expose to users.
pub use attributes::*;
pub use element::*;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;
pub use nodes::*;
pub use non_keyed_list::*;

// This is currently created by both `Nodes::Iter::update_text()` and `Nodes::Iter::static_text()`
pub struct Text {
    text: String,
    ws_node: web_sys::Node,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            ws_node: self
                .ws_node
                .clone_node_with_deep(false)
                .expect_throw("Unable to clone a web_sys::Node"),
        }
    }
}

impl Text {
    /// Create a text node from an expression
    pub fn new(text: &str) -> Self {
        let ws_node: web_sys::Node = crate::utils::document().create_text_node(text).into();
        Self {
            text: text.to_string(),
            ws_node,
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(&self.ws_node)
            .expect_throw("Unable to append a child Text to its expected parent");
    }

    pub fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
        parent
            .insert_before(&self.ws_node, next_sibling)
            .expect_throw("Unable to insert a child Text to its expected parent");
    }

    pub fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(&self.ws_node)
            .expect_throw("Unable to remove a child Text from its parent");
    }

    /// Update the node if the given `text` is new
    pub fn update_text(&mut self, text: &str) {
        if self.text != text {
            self.text = text.to_string();
            self.ws_node.set_text_content(Some(&self.text));
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ElementStatus {
    JustCreated,
    Existing,
    JustCloned,
}

pub trait RawWrapper<C: crate::component::Component> {
    fn ws_element(&self) -> &web_sys::Element;
    fn mounted(&self) {}
}

#[derive(Copy, Clone)]
pub enum ListElementCreation {
    Clone,
    New,
}

impl ListElementCreation {
    pub fn use_template(&self) -> bool {
        match self {
            Self::Clone => true,
            Self::New => false,
        }
    }
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
