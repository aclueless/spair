use std::ops::Deref;

use text::{Text, WsText};
use wasm_bindgen::{JsCast, UnwrapThrowExt, closure::Closure};
use web_sys::{
    DocumentFragment, HtmlInputElement, HtmlOptionElement, HtmlSelectElement, HtmlTemplateElement,
    HtmlTextAreaElement,
};

use crate::events::EventListener;
use crate::helper::{self, InputElementFromCurrentInputEvent};
use crate::{component::CallbackArg, routing::Route};

pub mod text;

pub trait WsNodeFns {
    fn get_ws_node_ref(&self) -> &web_sys::Node;

    fn first_node(&self) -> web_sys::Node {
        self.get_ws_node_ref()
            .first_child()
            .expect_throw("No first child node")
    }

    fn next_sibling_node(&self) -> web_sys::Node {
        self.get_ws_node_ref()
            .next_sibling()
            .expect_throw("No next sibling")
    }

    fn first_ws_node(&self) -> WsNode {
        self.first_node().into()
    }

    fn next_sibling_ws_node(&self) -> WsNode {
        self.next_sibling_node().into()
    }

    fn first_ws_text(&self) -> WsText {
        self.first_node().into()
    }

    fn next_sibling_ws_text(&self) -> WsText {
        self.next_sibling_node().into()
    }

    fn first_text(&self) -> Text {
        self.first_node().into()
    }

    fn next_sibling_text(&self) -> Text {
        self.next_sibling_node().into()
    }

    fn first_ws_element(&self) -> WsElement {
        self.first_node().into()
    }

    fn next_sibling_ws_element(&self) -> WsElement {
        self.next_sibling_node().into()
    }
}

impl WsNodeFns for web_sys::Node {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        self
    }
}

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

    pub fn fragment(&self) -> DocumentFragment {
        self.0.content()
    }

    pub(crate) fn fragment_clone(&self) -> DocumentFragment {
        self.0
            .content()
            .clone_node_with_deep(true)
            .expect("Clone a DocumentFragment")
            .unchecked_into()
    }
}
impl WsNodeFns for DocumentFragment {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        self
    }
}

pub struct WsNode(web_sys::Node);
impl From<web_sys::Node> for WsNode {
    fn from(value: web_sys::Node) -> Self {
        Self(value)
    }
}
impl WsNodeFns for WsNode {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        &self.0
    }
}

#[derive(Clone)]
pub struct WsElement(web_sys::Element);
impl From<web_sys::Node> for WsElement {
    fn from(value: web_sys::Node) -> Self {
        Self(value.unchecked_into())
    }
}
impl WsNodeFns for WsElement {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        &self.0
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
impl WsNodeFns for Element {
    fn get_ws_node_ref(&self) -> &web_sys::Node {
        &self.element.0
    }
}

enum Attribute {
    Bool(bool),
    I32(i32),
    Str(String),
    OptionString(Option<String>),
    EventListener(Box<dyn EventListener>),
}

impl WsElement {
    pub fn create_element(tag: &str) -> Self {
        Self(helper::create_element(tag))
    }

    pub fn create_element_with_capacity(self, capacity: usize) -> Element {
        Element {
            element: self,
            attributes: Vec::with_capacity(capacity),
        }
    }

    pub fn clear_text_content(&self) {
        self.0.set_text_content(None);
    }

    pub fn set_id(&self, id: &str) {
        self.0.set_id(id);
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

    pub fn insert_new_node_before_a_node(
        &self,
        new_node: &impl WsNodeFns,
        next_sibling: Option<&impl WsNodeFns>,
    ) {
        if let Err(e) = self.0.insert_before(
            new_node.get_ws_node_ref(),
            next_sibling.map(|v| v.get_ws_node_ref()),
        ) {
            log::error!("Error on inserting a new node into the child list: {e:?}");
        };
    }

    pub fn remove_child(&self, child: &impl WsNodeFns) {
        if let Err(e) = self.0.remove_child(child.get_ws_node_ref()) {
            log::error!("Error on removing child node: {e:?}");
        }
    }

    pub fn set_bool_attribute(&self, name: &str, value: bool) {
        let name = wasm_bindgen::intern(name);
        if value {
            if let Err(e) = self.0.set_attribute(name, "") {
                log::error!("Error on setting a boolean attribute ``{name}`: {e:?}`");
            }
        } else if let Err(e) = self.0.remove_attribute(name) {
            log::error!("Error on removing a boolean attribute ``{name}`: {e:?}`");
        }
    }

    pub fn set_str_attribute(&self, name: &str, value: &str) {
        let name = wasm_bindgen::intern(name);
        if let Err(e) = self.0.set_attribute(name, value) {
            log::error!("Error on setting an attributel {name}={value}: {e:?}");
        }
    }

    pub fn href_with_routing(&self, route: &impl Route) {
        let name = wasm_bindgen::intern("href");
        self.set_str_attribute(name, &route.url());
    }

    pub fn add_click_event_to_handle_routing(&self) {
        crate::routing::add_routing_handler(self);
    }

    pub fn unsafely_set_inner_html(&self, value: &str) {
        self.0.set_inner_html(value);
    }

    pub fn add_class(&self, class_name: &str) {
        let class_name = wasm_bindgen::intern(class_name);
        if let Err(e) = self.0.class_list().add_1(class_name) {
            log::error!("Error on adding a class name: {e:?}");
        }
    }

    fn remove_class(&self, class_name: &str) {
        let class_name = wasm_bindgen::intern(class_name);
        if let Err(e) = self.0.class_list().remove_1(class_name) {
            log::error!("Error on removing a class named `{class_name}`: {e:?}");
        }
    }

    pub fn class_if(&self, condition: bool, class_name: &str) {
        if condition {
            let class_name = wasm_bindgen::intern(class_name);
            self.add_class(class_name);
        }
    }

    fn add_or_remove_class(&self, condition: bool, class_name: &str) {
        let class_name = wasm_bindgen::intern(class_name);
        if condition {
            self.add_class(class_name);
        } else {
            self.remove_class(class_name);
        }
    }

    fn set_select_value_str(&self, value: &str) {
        self.0.unchecked_ref::<HtmlSelectElement>().set_value(value);
    }

    fn set_select_option_value(&self, value: Option<&str>) {
        match value {
            Some(value) => self.set_select_value_str(value),
            None => self.set_select_value_str(""),
        }
    }

    pub fn set_input_checked(&self, value: bool) {
        self.0
            .unchecked_ref::<HtmlInputElement>()
            .set_checked(value);
    }

    pub fn set_input_value(&self, value: &str) {
        self.0.unchecked_ref::<HtmlInputElement>().set_value(value);
    }

    pub fn set_textarea_value(&self, value: &str) {
        self.0
            .unchecked_ref::<HtmlTextAreaElement>()
            .set_value(value);
    }
    pub fn set_select_value(&self, value: impl SelectElementValue) {
        value.create(self);
    }

    pub fn set_option_value(&self, value: &str) {
        self.0.unchecked_ref::<HtmlOptionElement>().set_value(value);
    }
}

impl Element {
    pub fn ws_element(&self) -> &WsElement {
        &self.element
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

    fn swap_new_str_value(&mut self, index: usize, new_value: &str) -> StringChange {
        match self.attributes.get_mut(index) {
            Some(Attribute::Str(current_value)) => {
                if *current_value != new_value {
                    let mut temp_value = new_value.to_string();
                    std::mem::swap(current_value, &mut temp_value);
                    StringChange::OldValue(temp_value)
                } else {
                    StringChange::NoChange
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.attributes.push(Attribute::Str(new_value.to_string()));
                    StringChange::FirstTime
                } else {
                    log::error!(
                        "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
                        self.attributes.len()
                    );
                    StringChange::NoChange
                }
            }
            _ => {
                log::error!("Internal error: Attribute at index = {index} is not a String");
                StringChange::NoChange
            }
        }
    }

    fn is_new_option_str_value(&mut self, index: usize, new_value: Option<&str>) -> bool {
        match self.attributes.get_mut(index) {
            Some(Attribute::OptionString(current_value)) => {
                if current_value.as_deref() != new_value {
                    *current_value = new_value.map(|v| v.to_string());
                    true
                } else {
                    false
                }
            }
            None => {
                if self.attributes.len() == index {
                    self.attributes
                        .push(Attribute::OptionString(new_value.map(|v| v.to_string())));
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
                log::error!("Internal error: Attribute at index = {index} is not an OptionString");
                false
            }
        }
    }

    pub fn set_bool_attribute_at_index(&mut self, index: usize, name: &str, new_value: bool) {
        if self.is_new_bool_value(index, new_value) {
            self.element.set_bool_attribute(name, new_value);
        }
    }

    pub fn set_i32_attribute_at_index(&mut self, index: usize, name: &str, value: i32) {
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

    pub fn set_str_attribute_at_index(&mut self, index: usize, name: &str, value: &str) {
        if self.is_new_str_value(index, value) {
            self.element.set_str_attribute(name, value);
        }
    }

    pub fn set_string_attribute_at_index(&mut self, index: usize, name: &str, value: String) {
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

    pub fn update_class(&mut self, index: usize, class_name: &str) {
        match self.swap_new_str_value(index, class_name) {
            StringChange::FirstTime => self.element.add_class(class_name),
            StringChange::OldValue(old_class) => {
                self.element.remove_class(old_class.as_str());
                self.element.add_class(class_name);
            }
            StringChange::NoChange => {}
        }
    }

    pub fn class_if_at_index(&mut self, index: usize, condition: bool, class_name: &str) {
        if self.is_new_bool_value(index, condition) {
            self.element.add_or_remove_class(condition, class_name);
        }
    }

    pub fn class_or_at_index(
        &mut self,
        index: usize,
        condition: bool,
        first_class_name: &str,
        second_class_name: &str,
    ) {
        if self.is_new_bool_value(index, condition) {
            self.element
                .add_or_remove_class(condition, first_class_name);
            self.element
                .add_or_remove_class(!condition, second_class_name);
        }
    }

    pub fn href_with_routing_at_index(&mut self, index: usize, route: &impl Route) {
        self.set_string_attribute_at_index(index, "href", route.url());
    }

    pub fn set_input_checked_at_index(&mut self, index: usize, value: bool) {
        if self.is_new_bool_value(index, value) {
            self.set_input_checked(value);
        }
    }

    pub fn set_input_value_at_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.set_input_value(value);
        }
    }

    pub fn set_textarea_value_at_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.set_textarea_value(value);
        }
    }

    fn set_str_as_select_value_at_index(&mut self, index: usize, value: Option<&str>) {
        // if self.is_new_option_str_value(index, value) {
        //     self.element.set_select_option_value(value);
        // }
        //
        // An html select element always select the first option,
        // but I do not want it.
        // This is a hack to workaround it now.
        let _ = self.is_new_option_str_value(index, None);
        self.element.set_select_option_value(value);
    }

    fn set_string_as_select_value_at_index(&mut self, index: usize, new_value: Option<String>) {
        let _ = self.is_new_option_str_value(index, None);
        self.element.set_select_option_value(new_value.as_deref());
        // match self.attributes.get_mut(index) {
        //     Some(Attribute::OptionString(current_value)) => {
        //         if *current_value != new_value {
        //             *current_value = new_value;
        //             self.element
        //                 .set_select_option_value(current_value.as_deref());
        //         }
        //     }
        //     None => {
        //         if self.attributes.len() == index {
        //             self.element.set_select_option_value(new_value.as_deref());
        //             self.attributes.push(Attribute::OptionString(new_value));
        //         } else {
        //             log::error!(
        //                 "Internal error: A new attribute expected being added at the end of the list (index = {}), but the given index = {index}",
        //                 self.attributes.len()
        //             );
        //         }
        //     }
        //     _ => {
        //         log::error!("Internal error: Attribute at index = {index} is not an OptionString");
        //     }
        // }
    }

    pub fn set_select_value_at_index(&mut self, index: usize, value: impl SelectElementValue) {
        value.update(index, self);
    }

    pub fn set_option_value_at_index(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.set_option_value(value);
        }
    }

    pub fn unsafely_set_inner_html(&mut self, index: usize, value: &str) {
        if self.is_new_str_value(index, value) {
            self.element.unsafely_set_inner_html(value);
        }
    }
}

enum StringChange {
    FirstTime,
    OldValue(String),
    NoChange,
}

pub trait SelectElementValue {
    fn create(self, element: &WsElement);
    fn update(self, index: usize, element: &mut Element);
}

impl SelectElementValue for &str {
    fn create(self, element: &WsElement) {
        element.set_select_value_str(self);
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_str_as_select_value_at_index(index, Some(self));
    }
}

impl SelectElementValue for &String {
    fn create(self, element: &WsElement) {
        element.set_select_value_str(self);
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_str_as_select_value_at_index(index, Some(self));
    }
}

impl SelectElementValue for String {
    fn create(self, element: &WsElement) {
        element.set_select_value_str(&self);
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_string_as_select_value_at_index(index, Some(self));
    }
}

impl SelectElementValue for Option<&str> {
    fn create(self, element: &WsElement) {
        if let Some(value) = self {
            element.set_select_value_str(value);
        }
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_str_as_select_value_at_index(index, self);
    }
}

impl SelectElementValue for Option<&String> {
    fn create(self, element: &WsElement) {
        if let Some(value) = self {
            element.set_select_value_str(value);
        }
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_str_as_select_value_at_index(index, self.map(|v| v.as_str()));
    }
}

impl SelectElementValue for Option<String> {
    fn create(self, element: &WsElement) {
        if let Some(value) = self {
            element.set_select_value_str(value.as_str());
        }
    }
    fn update(self, index: usize, element: &mut Element) {
        element.set_string_as_select_value_at_index(index, self);
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
                self.$EventArgType(index, stringify!($event_name), callback);
            }
            )+
        }
    )+};
}
create_event_methods! {
    ClipboardEvent { copy cut paste }
    InputEvent { beforeinput input }
    Event { change }
    FocusEvent { blur focus focusin focusout }
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
                    let input = input_event.current_target_as_input();
                    callback.call(input.value());
                },
            )),
        );
    }

    pub fn input_checked(&mut self, index: usize, callback: CallbackArg<bool>) {
        self.add_event_listener(
            index,
            "input",
            Box::new(Closure::<dyn Fn(web_sys::InputEvent)>::new(
                move |input_event: web_sys::InputEvent| {
                    let input = input_event.current_target_as_input();
                    callback.call(input.checked());
                },
            )),
        );
    }
}
