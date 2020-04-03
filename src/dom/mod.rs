use wasm_bindgen::UnwrapThrowExt;

pub mod attribute_types;
mod attributes;
mod element;
mod nodes;

use attribute_types::AsStr;

// All items from these modules are visible here and in the super module (`dom`),
// but `dom` are private, the `lib.rs` must selectively export items from `dom`
// to expose to users.
pub use attributes::*;
pub use element::*;
pub use nodes::*;

// This is currently created by both `Nodes::Iter::update_text()` and `Nodes::Iter::static_text()`
struct Text {
    text: String,
    ws_node: web_sys::Node,
}

impl Clone for Text {
    fn clone(&self) -> Self {
        Self {
            text: String::new(),
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

    pub fn clear(&self, parent: &web_sys::Node) {
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

// A better name? Context?
pub struct Extra<'a, C> {
    pub comp: &'a crate::component::Comp<C>,
    pub status: ElementStatus,
    pub index: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ElementStatus {
    JustCreated,
    Existing,
    JustCloned,
}
