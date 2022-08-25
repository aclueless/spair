#[cfg(feature = "queue-render")]
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::{AttributeValueList, ElementType, Nodes, ParentAndChild};

#[derive(Debug)]
pub struct Element {
    ws_element: WsElement,
    attributes: AttributeValueList,
    nodes: Nodes,
    #[cfg(feature = "queue-render")]
    unmounted: Rc<Cell<bool>>,
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let ws_element = self.ws_element.shadow_clone();
        let nodes = self.nodes.clone();
        nodes.append_to(ws_element.ws_node());

        Self {
            ws_element,
            nodes,
            attributes: self.attributes.clone(),
            #[cfg(feature = "queue-render")]
            unmounted: Rc::new(Cell::new(false)),
        }
    }
}

impl ParentAndChild for Element {
    fn ws_node(&self) -> &web_sys::Node {
        self.ws_element.ws_node()
    }
}

impl Element {
    pub fn new_ns(ns: &str, tag: &str) -> Self {
        Self {
            ws_element: WsElement::new(ns, tag),
            attributes: Default::default(),
            nodes: Default::default(),
            #[cfg(feature = "queue-render")]
            unmounted: Rc::new(Cell::new(false)),
        }
    }

    pub fn from_ws_element(ws_element: web_sys::Element) -> Self {
        Self {
            ws_element: WsElement {
                element_type: ws_element.tag_name().to_ascii_lowercase().as_str().into(),
                ws_element,
            },
            attributes: Default::default(),
            nodes: Default::default(),
            #[cfg(feature = "queue-render")]
            unmounted: Rc::new(Cell::new(false)),
        }
    }

    pub fn mark_as_unmounted(&self) {
        #[cfg(feature = "queue-render")]
        self.unmounted.set(true);
    }
    #[cfg(feature = "queue-render")]
    pub fn unmounted(&self) -> Rc<Cell<bool>> {
        self.unmounted.clone()
    }

    // This is intended to use with child component
    // pub fn replace_ws_element(&mut self, ws_element: web_sys::Element) {
    //     self.ws_element = ws_element;
    //     self.nodes.append_to(self.ws_element.as_ref());
    // }

    pub fn is_empty(&self) -> bool {
        self.nodes.count() == 0
    }

    pub fn ws_element(&self) -> &WsElement {
        &self.ws_element
    }

    pub fn ws_node_and_nodes_mut(&mut self) -> (&web_sys::Node, &mut Nodes) {
        (self.ws_element.as_ref(), &mut self.nodes)
    }

    pub fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.ws_element.ws_element.unchecked_ref()
    }

    pub fn element_type(&self) -> ElementType {
        self.ws_element.element_type
    }

    pub fn attribute_list_mut(&mut self) -> &mut AttributeValueList {
        &mut self.attributes
    }

    #[cfg(test)]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut Nodes {
        &mut self.nodes
    }
}

// This is just a wrapper around web_sys::Element with some methods on it.
// WsElement is made to use both in regular spair and queue-render spair.
#[derive(Debug, Clone)]
pub struct WsElement {
    ws_element: web_sys::Element,
    element_type: ElementType,
}

pub trait AttributeValueAsString {
    fn to_string(self) -> String;
}

macro_rules! impl_string_attribute {
    ($($TypeName:ident)+) => {
        $(
            impl AttributeValueAsString for $TypeName {
                fn to_string(self) -> String {
                    ToString::to_string(&self)
                }
            }
        )+
    };
}

impl_string_attribute! { i32 u32 f64 }

impl WsElement {
    pub fn new(namespace: &str, tag: &str) -> Self {
        Self {
            ws_element: crate::utils::document()
                .create_element_ns(Some(namespace), tag)
                .expect_throw("dom::element::WsElement::new"),
            element_type: tag.into(),
        }
    }

    // A quick fix for keyed_list to work with WsElement. keyed_list is broken after intruducing
    // WsElement. keyed_list does not make use of WsElement yet.
    pub fn into_inner(self) -> web_sys::Element {
        self.ws_element
    }

    // A quick fix for keyed_list to work with WsElement. See into_inner for more.
    pub fn as_ref(&self) -> &web_sys::Element {
        &self.ws_element
    }

    pub fn ws_node(&self) -> &web_sys::Node {
        self.ws_element.as_ref()
    }

    pub fn ws_event_target(&self) -> &web_sys::EventTarget {
        self.ws_element.as_ref()
    }

    pub fn unchecked_ref<T: JsCast>(&self) -> &T {
        self.ws_element.unchecked_ref::<T>()
    }

    pub fn unchecked_into<T: JsCast>(&self) -> T {
        self.ws_element.clone().unchecked_into::<T>()
    }

    fn shadow_clone(&self) -> Self {
        Self {
            ws_element: self
                .ws_element
                .clone_node_with_deep(false)
                .expect_throw("render::element::WsElement::clone")
                .unchecked_into(),
            element_type: self.element_type,
        }
    }

    pub fn set_id(&self, id: &str) {
        self.ws_element.set_id(id);
    }

    pub fn set_text_content(&self, text: Option<&str>) {
        self.ws_element.set_text_content(text);
    }

    pub fn set_str_attribute(&self, attribute_name: &str, attribute_value: &str) {
        self.ws_element
            .set_attribute(attribute_name, attribute_value)
            .expect_throw("dom::element::WsElement::set_str_attribute");
    }

    pub fn remove_attribute(&self, attribute_name: &str) {
        self.ws_element
            .remove_attribute(attribute_name)
            .expect_throw("dom::element::WsElement::remove_attribute");
    }

    pub fn set_attribute<T: AttributeValueAsString>(
        &self,
        attribute_name: &str,
        attribute_value: T,
    ) {
        self.set_str_attribute(attribute_name, &attribute_value.to_string());
    }

    pub fn set_bool_attribute(&self, name: &str, value: bool) {
        if value {
            self.set_str_attribute(name, "");
        } else {
            self.remove_attribute(name);
        }
    }

    pub fn add_class(&self, class_name: &str) {
        self.ws_element
            .class_list()
            .add_1(class_name)
            .expect_throw("dom::element::WsElement::add_class");
    }

    pub fn remove_class(&self, class_name: &str) {
        self.ws_element
            .class_list()
            .remove_1(class_name)
            .expect_throw("dom::element::WsElement::remove_class");
    }

    // return `true` if the element is a <select>
    // If it's in queue render mode, the value is always set, users
    // must make sure that the value is set after the children
    // of <select> are added.
    #[must_use = "Make sure that the return value is handled if queue_render = false"]
    pub fn set_value(&self, value: &str, queue_render: bool) -> bool {
        match self.element_type {
            ElementType::Input => {
                let input = self.ws_element.unchecked_ref::<web_sys::HtmlInputElement>();
                input.set_value(value);
            }
            ElementType::Select => {
                if queue_render {
                    let select = self
                        .ws_element
                        .unchecked_ref::<web_sys::HtmlSelectElement>();
                    select.set_value(value);
                }
                return true;
            }
            ElementType::TextArea => {
                let text_area = self
                    .ws_element
                    .unchecked_ref::<web_sys::HtmlTextAreaElement>();
                text_area.set_value(value);
            }
            ElementType::Option => {
                let option = self
                    .ws_element
                    .unchecked_ref::<web_sys::HtmlOptionElement>();
                option.set_value(value);
            }
            ElementType::Other => {
                log::warn!(
                    ".value() is called on an element that is not <input>, <select>, <option>, <textarea>"
                );
            }
        }
        false
    }

    #[allow(clippy::ptr_arg)]
    pub fn set_value_for_qr(&self, value: &String) {
        let _user = self.set_value(value, true);
    }

    pub fn set_value_for_qr_optional(&self, value: &Option<String>) {
        match value {
            Some(value) => {
                let _user = self.set_value(value, true);
            }
            None => self.set_selected_index(-1),
        }
    }

    pub fn set_selected_index(&self, index: i32) {
        match self.element_type {
            ElementType::Select => {
                let select = self
                    .ws_element
                    .unchecked_ref::<web_sys::HtmlSelectElement>();
                select.set_selected_index(index);
            }
            _ => {
                log::warn!(".set_selected_index() is called on an element that is not a <select>");
            }
        }
    }

    pub fn set_selected_index_ref(&self, index: &usize) {
        self.set_selected_index(*index as i32);
    }

    pub fn set_selected_index_optional(&self, index: &Option<usize>) {
        match index {
            Some(index) => self.set_selected_index(*index as i32),
            None => self.set_selected_index(-1),
        }
    }

    pub fn checked_ref(&self, value: &bool) {
        self.checked(*value);
    }

    pub fn checked(&self, value: bool) {
        if self.element_type == ElementType::Input {
            let input = self.ws_element.unchecked_ref::<web_sys::HtmlInputElement>();
            input.set_checked(value);
        } else {
            log::warn!(".checked() is called on an element that is not an <input>");
        }
    }

    pub fn scroll_to_view_with_bool(&self, align_to_top: bool) {
        self.ws_element.scroll_into_view_with_bool(align_to_top);
    }

    pub fn scroll_to_view_with_options(&self, options: &web_sys::ScrollIntoViewOptions) {
        self.ws_element
            .scroll_into_view_with_scroll_into_view_options(options);
    }
}
