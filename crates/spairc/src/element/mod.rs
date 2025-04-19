use std::{
    fmt::Display,
    ops::{Deref, DerefMut, Not},
};

use values::{Value, ValueChanged};
use wasm_bindgen::{JsCast, UnwrapThrowExt, closure::Closure};
use web_sys::{
    HtmlInputElement, HtmlOptionElement, HtmlSelectElement, HtmlTemplateElement,
    HtmlTextAreaElement,
};

use crate::{CallbackArg, events::EventListener, helper, routing::Route};

mod values;

pub struct TemplateElement(HtmlTemplateElement);
impl TemplateElement {
    pub fn new(html: &str) -> Self {
        let template = crate::helper::DOCUMENT
            .with(|document| document.create_element("template"))
            .expect_throw("Error on creating a template node");
        let template: HtmlTemplateElement = template.unchecked_into();
        template.set_inner_html(html);
        Self(template)
    }

    pub fn create_element(&self, capacity: usize) -> Element {
        let element = self
            .0
            .content()
            .first_child()
            .map(|node| {
                node.clone_node_with_deep(true)
                    .expect_throw("Unable to clone the template node")
            })
            .expect_throw("No element in the template")
            .unchecked_into();
        Element {
            element: WsElement(element),
            attributes: Vec::with_capacity(capacity),
        }
    }
}

pub struct WsNodeRef<'a>(&'a web_sys::Node);
impl WsNodeRef<'_> {
    fn first_node(&self) -> web_sys::Node {
        self.0.first_child().expect_throw("No first child node")
    }

    fn next_sibling_node(&self) -> web_sys::Node {
        self.0.next_sibling().expect_throw("No next sibling")
    }

    pub fn first_ws_node(&self) -> WsNode {
        self.first_node().into()
    }

    pub fn next_sibling_ws_node(&self) -> WsNode {
        self.next_sibling_node().into()
    }

    pub fn first_ws_text(&self) -> WsText {
        self.first_node().into()
    }

    pub fn next_sibling_ws_text(&self) -> WsText {
        self.next_sibling_node().into()
    }

    pub fn first_text(&self) -> Text {
        self.first_node().into()
    }

    pub fn next_sibling_text(&self) -> Text {
        self.next_sibling_node().into()
    }

    pub fn first_ws_element(&self) -> WsElement {
        self.first_node().into()
    }

    pub fn next_sibling_ws_element(&self) -> WsElement {
        self.next_sibling_node().into()
    }
}

pub struct WsNode(web_sys::Node);

impl From<web_sys::Node> for WsNode {
    fn from(value: web_sys::Node) -> Self {
        Self(value)
    }
}

impl Deref for WsNode {
    type Target = web_sys::Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WsNode {
    pub fn ws_node_ref(&self) -> WsNodeRef {
        WsNodeRef(self.0.as_ref())
    }
}

pub struct WsText(web_sys::Text);

impl From<web_sys::Node> for WsText {
    fn from(value: web_sys::Node) -> Self {
        Self(value.unchecked_into())
    }
}

impl WsText {
    pub fn split_text(&self, off_set: u32) {
        if let Err(e) = self.0.split_text(off_set) {
            log::error!("{e:?}");
        }
    }

    pub fn clear_text_content(&self) {
        self.0.set_text_content(None);
    }

    fn set_text_content(&self, text: &str) {
        self.0.set_text_content(Some(text));
    }

    pub fn set_text(&self, text: impl RenderAsText) {
        text.create(self);
    }

    pub fn ws_node_ref(&self) -> WsNodeRef {
        WsNodeRef(self.0.as_ref())
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

impl Deref for Text {
    type Target = web_sys::Text;

    fn deref(&self) -> &Self::Target {
        &self.ws_text.0
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

    fn update_with_default(&mut self, default: &str) {
        if matches!(&self.value, Value::Default).not() {
            self.value = Value::Default;
            self.ws_text.set_text_content(default);
        }
    }

    pub fn update<T: RenderAsText>(&mut self, value: T) {
        value.update(self);
    }

    pub fn ws_node_ref(&self) -> WsNodeRef {
        WsNodeRef(self.ws_text.0.as_ref())
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

#[derive(Clone)]
pub struct WsElement(web_sys::Element);

impl Deref for WsElement {
    type Target = web_sys::Element;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<web_sys::Node> for WsElement {
    fn from(value: web_sys::Node) -> Self {
        Self(value.unchecked_into())
    }
}

impl WsElement {
    pub fn ws_element(&self) -> &Self {
        self
    }

    pub fn web_sys_node(&self) -> &web_sys::Node {
        &self.0
    }

    pub fn clear_text_content(&self) {
        self.0.set_text_content(None);
    }

    pub fn replace_at_element_id(&self, element_id: &str) {
        let Some(element) = helper::get_element_by_id(element_id) else {
            log::error!("Unable to find element by id: {element_id}");
            return;
        };
        if let Err(e) = element.replace_with_with_node_1(&self.0) {
            log::error!("Error on replacing at element id: {element_id}: {e:?}");
        }
        self.set_id(element_id);
    }

    pub fn append_to_body(&self) -> WsElement {
        let body = crate::helper::get_body();
        if let Err(e) = body.append_with_node_1(&self.0) {
            log::error!("Error on appending to body: {e:?}");
        };
        body.unchecked_into::<web_sys::Node>().into()
    }

    pub(crate) fn add_event_listener(&self, name: &str, listener: &dyn EventListener) {
        let name = wasm_bindgen::intern(name);
        if let Err(e) = self
            .0
            .add_event_listener_with_callback(name, listener.js_function())
        {
            log::error!("Error on adding event listener for `{name}`: {e:?}");
        }
    }

    fn remove_event_listener(&self, name: &str, listener: &dyn EventListener) {
        let name = wasm_bindgen::intern(name);
        if let Err(e) = self
            .0
            .remove_event_listener_with_callback(name, listener.js_function())
        {
            log::error!("Error on removing event listener for `{name}`: {e:?}");
        }
    }

    pub fn set_bool_attribute(&self, name: &str, value: bool) {
        let name = wasm_bindgen::intern(name);
        if value {
            if let Err(e) = self.0.remove_attribute(name) {
                log::error!("Error on setting a boolean attribute ``{name}`: {e:?}`");
            }
        } else if let Err(e) = self.0.set_attribute(name, "") {
            log::error!("Error on removing a boolean attribute ``{name}`: {e:?}`");
        }
    }

    pub fn set_str_attribute(&self, name: &str, value: &str) {
        let name = wasm_bindgen::intern(name);
        if let Err(e) = self.0.set_attribute(name, value) {
            log::error!("Error on setting an attributel {name}={value}: {e:?}");
        }
    }

    pub fn ws_node_ref(&self) -> WsNodeRef {
        WsNodeRef(self.0.as_ref())
    }

    pub fn insert_new_node_before_a_node(
        &self,
        new_node: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if let Err(e) = self.0.insert_before(new_node, next_sibling) {
            log::error!("Error on inserting a new node into the child list: {e:?}");
        };
    }

    pub fn append_new_node(&self, new_node: &web_sys::Node) {
        if let Err(e) = self.0.append_child(new_node) {
            log::error!("Error on appending a node to its parent: {e:?}");
        };
    }

    fn add_or_remove_class(&self, condition: bool, class_name: &str) {
        if condition {
            if let Err(e) = self.0.class_list().add_1(class_name) {
                log::error!("Error on adding a class named `{class_name}`: {e:?}");
            }
        } else if let Err(e) = self.0.class_list().remove_1(class_name) {
            log::error!("Error on removing a class named `{class_name}`: {e:?}");
        }
    }

    pub fn create_element_with_capacity(self, capacity: usize) -> Element {
        Element {
            element: self,
            attributes: Vec::with_capacity(capacity),
        }
    }

    pub fn remove_child(&self, child: &web_sys::Node) {
        if let Err(e) = self.0.remove_child(child) {
            log::error!("Error on removing child node: {e:?}");
        }
    }

    pub fn class(&self, class_name: &str) {
        if let Err(e) = self.0.class_list().add_1(class_name) {
            log::error!("Error on adding a class name: {e:?}");
        }
    }

    pub fn href_with_routing(&self, route: &impl Route) {
        self.set_str_attribute("href", &route.url());
    }

    pub fn add_click_event_to_handle_routing(&self) {
        crate::routing::add_routing_handler(self);
    }
}

pub struct Element {
    element: WsElement,
    attributes: Vec<Attribute>,
}

impl Deref for Element {
    type Target = WsElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.element
    }
}

enum Attribute {
    Bool(bool),
    I32(i32),
    Str(String),
    EventListener(Box<dyn EventListener>),
}

impl Element {
    pub fn with_html(html: &str, capacity: usize) -> Self {
        let template = TemplateElement::new(html);
        Self {
            element: WsElement(
                template
                    .0
                    .content()
                    .first_child()
                    .expect_throw("No element in the template")
                    .unchecked_into(),
            ),
            attributes: Vec::with_capacity(capacity),
        }
    }

    pub fn add_event_listener(
        &mut self,
        index: usize,
        name: &str,
        listener: Box<dyn EventListener>,
    ) {
        if let Some(old_listener) = self.attributes.get_mut(index) {
            if let Attribute::EventListener(old_listener) = old_listener {
                self.element
                    .remove_event_listener(name, old_listener.as_ref());
                self.element.add_event_listener(name, listener.as_ref());
                *old_listener = listener;
            } else {
                log::error!("Internal error: Attribute at index = {index} is not a event listener")
            }
        } else if self.attributes.len() == index {
            self.element.add_event_listener(name, listener.as_ref());
            self.attributes.push(Attribute::EventListener(listener));
        } else {
            log::error!(
                "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                self.attributes.len()
            );
        }
    }

    fn is_new_bool_value(&mut self, index: usize, new_value: bool) -> bool {
        match self.attributes.get_mut(index) {
            Some(Attribute::Bool(current_value)) => {
                if *current_value != new_value {
                    *current_value = new_value;
                    true
                } else {
                    false
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.attributes.push(Attribute::Bool(new_value));
                    true
                } else {
                    log::error!(
                        "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                        self.attributes.len()
                    );
                    false
                }
            }
            _ => {
                log::error!("Internal error: Attribute at index = {index} is not a bool");
                false
            }
        }
    }

    fn is_new_str_value(&mut self, index: usize, new_value: &str) -> bool {
        match self.attributes.get_mut(index) {
            Some(Attribute::Str(current_value)) => {
                if *current_value != new_value {
                    *current_value = new_value.to_string();
                    true
                } else {
                    false
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.attributes.push(Attribute::Str(new_value.to_string()));
                    true
                } else {
                    log::error!(
                        "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                        self.attributes.len()
                    );
                    false
                }
            }
            _ => {
                log::error!("Internal error: Attribute at index = {index} is not a String");
                false
            }
        }
    }

    pub fn set_bool_attribute_with_index(&mut self, index: usize, name: &str, new_value: bool) {
        if self.is_new_bool_value(index, new_value) {
            self.element.set_bool_attribute(name, new_value);
        }
    }

    pub fn set_i32_attribute_with_index(&mut self, index: usize, name: &str, value: i32) {
        match self.attributes.get_mut(index) {
            Some(Attribute::I32(current_value)) => {
                if *current_value != value {
                    *current_value = value;
                    self.element.set_str_attribute(name, &value.to_string());
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.attributes.push(Attribute::I32(value));
                    self.element.set_str_attribute(name, &value.to_string());
                } else {
                    log::error!(
                        "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                        self.attributes.len()
                    );
                }
            }
            _ => {
                log::error!("Internal error: Attribute at index = {index} is not a i32")
            }
        }
    }

    pub fn set_str_attribute_with_index(&mut self, index: usize, name: &str, value: &str) {
        // match self.attributes.get_mut(index) {
        //     Some(Attribute::Str(current_value)) => {
        //         if *current_value != value {
        //             *current_value = value.to_string();
        //             self.element.set_str_attribute(name, value);
        //         }
        //     }
        //     None => {
        //         if self.attributes.len() == index {
        //             self.attributes.push(Attribute::Str(value.to_string()));
        //             self.element.set_str_attribute(name, value);
        //         } else {
        //             log::error!("Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}", self.attributes.len());
        //         }
        //     }
        //     _ => {
        //         log::error!("Internal error: Attribute at index = {index} is not a i32")
        //     }
        // }
        if self.is_new_str_value(index, value) {
            self.element.set_str_attribute(name, value);
        }
    }

    pub fn set_string_attribute_with_index(&mut self, index: usize, name: &str, value: String) {
        match self.attributes.get_mut(index) {
            Some(Attribute::Str(current_value)) => {
                if *current_value != value {
                    *current_value = value;
                    self.element.set_str_attribute(name, current_value);
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.element.set_str_attribute(name, &value);
                    self.attributes.push(Attribute::Str(value));
                } else {
                    log::error!(
                        "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                        self.attributes.len()
                    );
                }
            }
            _ => {
                log::error!("Internal error: Attribute at index = {index} is not a i32")
            }
        }
    }

    pub fn ws_element(&self) -> &WsElement {
        &self.element
    }

    pub fn class_if_with_index(&mut self, index: usize, condition: bool, class_name: &str) {
        if self.is_new_bool_value(index, condition) {
            self.element.add_or_remove_class(condition, class_name);
        }
    }

    pub fn href_with_routing_with_index(&mut self, index: usize, route: &impl Route) {
        self.set_string_attribute_with_index(index, "href", route.url());
    }

    pub fn set_input_checked_with_index(&mut self, index: usize, value: bool) {
        if self.is_new_bool_value(index, value) {
            self.0
                .unchecked_ref::<HtmlInputElement>()
                .set_checked(value);
        }
    }

    pub fn set_input_value_with_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.0.unchecked_ref::<HtmlInputElement>().set_value(value);
        }
    }

    pub fn set_textarea_value_with_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.0
                .unchecked_ref::<HtmlTextAreaElement>()
                .set_value(value);
        }
    }

    pub fn set_select_value_with_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.0.unchecked_ref::<HtmlSelectElement>().set_value(value);
        }
    }

    pub fn set_option_value_with_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.0.unchecked_ref::<HtmlOptionElement>().set_value(value);
        }
    }
}

macro_rules! create_event_methods {
    ($($EventArgType:ident { $($event_name:ident)+ })+) => {$(
        impl Element {
            #[allow(non_snake_case)]
            fn $EventArgType(&mut self, index: usize, event_name: &str, callback: CallbackArg<web_sys::$EventArgType>) {
                self.add_event_listener(
                    index,
                    event_name,
                    Box::new(Closure::<dyn Fn(web_sys::$EventArgType)>::new(move |arg| callback.call(arg)))
                )
            }
            $(
            pub fn $event_name(&mut self, index: usize, callback: CallbackArg<web_sys::$EventArgType>) {
                // self.add_event_listener(
                //     index,
                //     stringify!($event_name),
                //     Box::new(Closure::<dyn Fn(web_sys::$EventArgType)>::new(move |arg| callback.call(arg)))
                // )
                self.$EventArgType(index, stringify!($event_name), callback);
            }
            )+
        }
    )+};
}

create_event_methods! {
    Event { change }
    FocusEvent { focus blur focusin focusout }
    KeyboardEvent { keydown }
    MouseEvent { click dblclick mousedown mouseup mouseenter mouseleave mousemove mouseover }
}

impl Element {
    pub fn input_string(&mut self, index: usize, callback: CallbackArg<String>) {
        self.add_event_listener(
            index,
            "input",
            Box::new(Closure::<dyn Fn(web_sys::InputEvent)>::new(
                move |input_event: web_sys::InputEvent| {
                    if let Some(input_element) = input_event
                        .current_target()
                        .map(|v| v.unchecked_into::<HtmlInputElement>())
                    {
                        callback.call(input_element.value());
                    };
                },
            )),
        );
    }
}
