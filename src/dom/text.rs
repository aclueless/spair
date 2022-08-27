use super::AChildNode;
use wasm_bindgen::UnwrapThrowExt;

pub struct TextNode {
    text: String,
    ws_node: web_sys::Node,
}

impl Clone for TextNode {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            ws_node: self.ws_node.clone_node_with_deep(false).expect_throw(
                "dom::text::TextNode::clone::self.ws_node.clone_node_with_deep(false)",
            ),
        }
    }
}

impl AChildNode for TextNode {
    fn ws_node(&self) -> &web_sys::Node {
        &self.ws_node
    }
}

pub trait IntoString: PartialEq<String> {
    fn into_string(self) -> String;
}

impl IntoString for &str {
    fn into_string(self) -> String {
        self.to_string()
    }
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}

impl TextNode {
    pub fn new(text: impl IntoString) -> Self {
        let text = text.into_string();
        let ws_node: web_sys::Node = crate::utils::document().create_text_node(&text).into();
        Self { text, ws_node }
    }

    /// Update the node if the given `text` is new
    pub fn update_text(&mut self, text: impl IntoString) {
        if text.ne(&self.text) {
            self.text = text.into_string();
            self.ws_node.set_text_content(Some(&self.text));
        }
    }

    #[cfg(test)]
    pub fn text(&self) -> &String {
        &self.text
    }
}
