use super::AChildNode;
use wasm_bindgen::UnwrapThrowExt;

pub trait InternalTextRender: PartialEq<TextNodeValue> + ToString {
    fn create_text_node_value(self) -> TextNodeValue;
    fn update(&self, ws_node: &web_sys::Node);
}

pub struct TextNode {
    value: TextNodeValue,
    ws_node: web_sys::Node,
}

impl Clone for TextNode {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
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

#[derive(Clone)]
pub enum TextNodeValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    ISize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    USize(usize),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
    String(String),
}

macro_rules! impl_text_render {
    ($($Var:ident($type:ty),)+) => {
        $(
        impl PartialEq<TextNodeValue> for $type {
            fn eq(&self, rhs: &TextNodeValue) -> bool {
                matches!(rhs, TextNodeValue::$Var(value) if value == self)
            }
        }
        impl InternalTextRender for $type {
            fn create_text_node_value(self) -> TextNodeValue {
                TextNodeValue::$Var(self)
            }
            fn update(&self, ws_node: &web_sys::Node) {
                let text = self.to_string();
                ws_node.set_text_content(Some(&text));
            }
        }
        )+
    };
}

impl_text_render! {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    ISize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    USize(usize),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
}

impl PartialEq<TextNodeValue> for &str {
    fn eq(&self, rhs: &TextNodeValue) -> bool {
        matches!(rhs, TextNodeValue::String(value) if value == self)
    }
}

impl PartialEq<TextNodeValue> for &String {
    fn eq(&self, rhs: &TextNodeValue) -> bool {
        matches!(rhs, TextNodeValue::String(value) if value == *self)
    }
}

impl PartialEq<TextNodeValue> for String {
    fn eq(&self, rhs: &TextNodeValue) -> bool {
        matches!(rhs, TextNodeValue::String(value) if value == self)
    }
}

impl InternalTextRender for &str {
    fn create_text_node_value(self) -> TextNodeValue {
        TextNodeValue::String(self.to_string())
    }

    fn update(&self, ws_node: &web_sys::Node) {
        ws_node.set_text_content(Some(self));
    }
}

impl InternalTextRender for &String {
    fn create_text_node_value(self) -> TextNodeValue {
        TextNodeValue::String(self.to_string())
    }

    fn update(&self, ws_node: &web_sys::Node) {
        ws_node.set_text_content(Some(self));
    }
}

impl InternalTextRender for String {
    fn create_text_node_value(self) -> TextNodeValue {
        TextNodeValue::String(self)
    }

    fn update(&self, ws_node: &web_sys::Node) {
        ws_node.set_text_content(Some(self));
    }
}

impl TextNode {
    pub fn new(value: TextNodeValue, ws_node: web_sys::Node) -> Self {
        Self { value, ws_node }
    }
    /// Update the node if the given `value` is different from the current value
    pub fn update_text(&mut self, value: impl InternalTextRender) {
        if value.ne(&self.value) {
            value.update(&self.ws_node);
            self.value = value.create_text_node_value();
        }
    }

    #[cfg(test)]
    pub fn test_string(&self) -> String {
        match &self.value {
            TextNodeValue::I8(v) => v.to_string(),
            TextNodeValue::I16(v) => v.to_string(),
            TextNodeValue::I32(v) => v.to_string(),
            TextNodeValue::I64(v) => v.to_string(),
            TextNodeValue::U8(v) => v.to_string(),
            TextNodeValue::U16(v) => v.to_string(),
            TextNodeValue::U32(v) => v.to_string(),
            TextNodeValue::U64(v) => v.to_string(),
            TextNodeValue::F32(v) => v.to_string(),
            TextNodeValue::F64(v) => v.to_string(),
            TextNodeValue::ISize(v) => v.to_string(),
            TextNodeValue::USize(v) => v.to_string(),
            TextNodeValue::Bool(v) => v.to_string(),
            TextNodeValue::String(v) => v.to_string(),
            TextNodeValue::Char(v) => v.to_string(),
        }
    }
}
