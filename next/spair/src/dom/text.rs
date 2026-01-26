use std::{fmt::Display, ops::Not};

use wasm_bindgen::JsCast;

use super::WsNodeFns;

pub struct WsText(web_sys::Text);
impl From<web_sys::Node> for WsText {
    fn from(value: web_sys::Node) -> Self {
        Self(value.unchecked_into())
    }
}
impl WsNodeFns for WsText {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        &self.0
    }
}

pub struct Text {
    ws_text: WsText,
    value: Value,
}
impl From<web_sys::Node> for Text {
    fn from(value: web_sys::Node) -> Self {
        Self {
            ws_text: WsText(value.unchecked_into()),
            value: Value::None,
        }
    }
}
impl WsNodeFns for Text {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        &self.ws_text.0
    }
}

pub trait RenderAsText {
    fn create(self, text: &WsText);
    fn update(self, text: &mut Text);
}

impl RenderAsText for &str {
    fn create(self, text: &WsText) {
        text.set_text_content(self);
    }
    fn update(self, text: &mut Text) {
        text.update_with_str(self);
    }
}

impl RenderAsText for &String {
    fn create(self, text: &WsText) {
        text.set_text_content(self);
    }
    fn update(self, text: &mut Text) {
        text.update_with_str(self);
    }
}

impl RenderAsText for String {
    fn create(self, text: &WsText) {
        text.set_text_content(&self);
    }
    fn update(self, text: &mut Text) {
        text.update_with_string(self);
    }
}

macro_rules! impl_render_as_text {
    ($($type_name:ident)+) => {
        $(
            impl RenderAsText for $type_name {
                fn create(self, text: &WsText) {
                    text.set_text_content(&self.to_string());
                }
                fn update(self, text: &mut Text) {
                    if self.check_value_changed(&mut text.value) {
                        text.ws_text.set_text_content(&self.to_string());
                    }
                }
            }

        )+
    };
}

impl_render_as_text!(isize i8 i16 i32 i64 i128 usize u8 u16 u32 u64 u128 f32 f64 bool char);

impl<T: ValueChanged + Display> RenderAsText for (Option<T>, &str) {
    fn create(self, text: &WsText) {
        if let Some(value) = self.0 {
            text.set_text_content(&value.to_string());
        } else {
            text.set_text_content(self.1);
        }
    }

    fn update(self, text: &mut Text) {
        if let Some(value) = self.0 {
            if value.check_value_changed(&mut text.value) {
                text.ws_text.set_text_content(&value.to_string());
            }
        } else {
            text.update_with_default(self.1);
        }
    }
}

impl RenderAsText for (Option<&str>, &str) {
    fn create(self, text: &WsText) {
        if let Some(value) = self.0 {
            text.set_text_content(value);
        } else {
            text.set_text_content(self.1);
        }
    }

    fn update(self, text: &mut Text) {
        if let Some(value) = self.0 {
            text.update_with_str(value);
        } else {
            text.update_with_default(self.1);
        }
    }
}

impl RenderAsText for (Option<&String>, &str) {
    fn create(self, text: &WsText) {
        (self.0.map(|v| v.as_str()), self.1).create(text);
    }

    fn update(self, text: &mut Text) {
        (self.0.map(|v| v.as_str()), self.1).update(text);
    }
}

impl RenderAsText for (Option<String>, &str) {
    fn create(self, text: &WsText) {
        if let Some(value) = self.0 {
            text.set_text_content(&value);
        } else {
            text.set_text_content(self.1);
        }
    }

    fn update(self, text: &mut Text) {
        if let Some(value) = self.0 {
            text.update_with_string(value);
        } else {
            text.update_with_default(self.1);
        }
    }
}

pub trait RenderOptionWithDefault<T> {
    fn or_default(self, default: &str) -> (Option<T>, &str);
}

impl<T> RenderOptionWithDefault<T> for Option<T> {
    fn or_default(self, default: &str) -> (Option<T>, &str) {
        (self, default)
    }
}

impl WsText {
    pub fn split_text(&self, off_set: u32) {
        if let Err(e) = self.0.split_text(off_set) {
            log::error!("{e:?}");
        }
    }

    pub fn set_text(&self, text: impl RenderAsText) {
        text.create(self);
    }

    fn set_text_content(&self, text: &str) {
        self.0.set_text_content(Some(text));
    }
}

impl Text {
    pub fn split_text(&self, off_set: u32) {
        self.ws_text.split_text(off_set);
    }

    fn update_with_str(&mut self, text: &str) {
        if let Value::String(old_value) = &mut self.value {
            if *old_value != text {
                self.ws_text.set_text_content(text);
                *old_value = text.to_string();
            }
        } else {
            self.ws_text.set_text_content(text);
            self.value = Value::String(text.to_string());
        }
    }

    fn update_with_string(&mut self, text: String) {
        if let Value::String(old_value) = &mut self.value {
            if *old_value != text {
                self.ws_text.set_text_content(&text);
                *old_value = text;
            }
        } else {
            self.ws_text.set_text_content(&text);
            self.value = Value::String(text);
        }
    }

    pub fn update<T: RenderAsText>(&mut self, value: T) {
        value.update(self);
    }

    fn update_with_default(&mut self, default: &str) {
        if matches!(&self.value, Value::Default).not() {
            self.value = Value::Default;
            self.ws_text.set_text_content(default);
        }
    }
}

pub enum Value {
    None,
    Default,
    Bool(bool),
    Char(char),
    Isize(isize),
    Usize(usize),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    F32(f32),
    F64(f64),
    String(String),
}

pub trait ValueChanged: Copy {
    fn check_value_changed(self, value: &mut Value) -> bool;
}

macro_rules! impl_value_changed {
    ($($Variant:ident $Type:ty)+) => {
        $(
        impl ValueChanged for $Type {
            fn check_value_changed(self, value: &mut Value) -> bool {
                if let Value::$Variant(old_value) = value {
                    if *old_value != self {
                        *old_value = self;
                        return true;
                    }
                }
                else{
                    *value = Value::$Variant(self);
                    return true;
                }
                false
            }
        }
        )+
    };
}

impl_value_changed! {
    Bool bool
    Char char
    Isize isize
    Usize usize
    I8 i8
    U8 u8
    I16 i16
    U16 u16
    I32 i32
    U32 u32
    I64 i64
    U64 u64
    I128 i128
    U128 u128
    F32 f32
    F64 f64
}
