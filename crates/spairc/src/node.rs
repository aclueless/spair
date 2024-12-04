pub struct TextNode {
    node: web_sys::Node,
    text: String,
}

impl TextNode {
    pub fn new(node: web_sys::Node, text: String) -> Self {
        node.set_text_content(Some(&text));
        Self { node, text }
    }

    pub fn update_with_str(&mut self, text: &str) {
        if self.text != text {
            self.node.set_text_content(Some(text));
            self.text = text.to_string();
        }
    }

    pub fn update_with_string(&mut self, text: String) {
        if self.text != text {
            self.node.set_text_content(Some(&text));
            self.text = text;
        }
    }
}
