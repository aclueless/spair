#[cfg(feature = "queue-render")]
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::{AttributeValueList, ElementType, Nodes, ParentAndChild};

#[derive(Debug)]
pub struct Element {
    element_type: ElementType,
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
            element_type: self.element_type,
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
            element_type: tag.into(),
            ws_element: WsElement::new(ns, tag),
            attributes: Default::default(),
            nodes: Default::default(),
            #[cfg(feature = "queue-render")]
            unmounted: Rc::new(Cell::new(false)),
        }
    }

    pub fn from_ws_element(ws_element: web_sys::Element) -> Self {
        Self {
            element_type: ws_element.tag_name().to_ascii_lowercase().as_str().into(),
            ws_element: WsElement(ws_element),
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
        (self.ws_element.0.as_ref(), &mut self.nodes)
    }

    pub fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.ws_element.0.unchecked_ref()
    }

    pub fn element_type(&self) -> ElementType {
        self.element_type
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

    // pub fn scroll_to_view_with_bool(&self, align_to_top: bool) {
    //     self.ws_element.scroll_to_view_with_bool(align_to_top);
    // }

    // pub fn scroll_to_view_with_options(&self, options: &web_sys::ScrollIntoViewOptions) {
    //     self.ws_element.scroll_to_view_with_options(options);
    // }

    // pub fn set_bool_attribute(&mut self, name: &str, value: bool) {
    //     self.ws_element.set_bool_attribute(name, value);
    // }

    // pub fn set_str_attribute(&mut self, name: &str, value: &str) {
    //     self.ws_element.set_str_attribute(name, value);
    // }

    // pub fn set_attribute<T: AttributeValueAsString>(&mut self, name: &str, value: T) {
    //     self.ws_element.set_attribute(name, value);
    // }

    // pub fn set_i32_attribute(&mut self, name: &str, value: i32) {
    //     self.ws_element()
    //         .set_attribute(name, &value.to_string())
    //         .expect_throw("dom::element::Element::set_i32_attribute::set");
    // }

    // pub fn set_u32_attribute(&mut self, name: &str, value: u32) {
    //     self.ws_element()
    //         .set_attribute(name, &value.to_string())
    //         .expect_throw("dom::element::Element::set_u32_attribute::set");
    // }

    // pub fn set_f64_attribute(&mut self, name: &str, value: f64) {
    //     self.ws_element()
    //         .set_attribute(name, &value.to_string())
    //         .expect_throw("dom::element::Element::set_f64_attribute::set");
    // }
}

// This is just a wrapper around web_sys::Element with some methods on it.
// WsElement is made to use both in regular spair and queue-render spair.
#[derive(Debug, Clone)]
pub struct WsElement(web_sys::Element);

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
        Self(crate::utils::document().create_element_ns(Some(namespace), tag).expect_throw(
            "dom::element::WsElement::new",
        ))
    }

    pub fn ws_node(&self) -> &web_sys::Node {
        self.0.as_ref()
    }

    pub fn ws_event_target(&self) -> &web_sys::EventTarget {
        self.0.as_ref()
    }

    pub fn unchecked_ref<T: JsCast>(&self) -> &T {
        self.0.unchecked_ref::<T>()
    }

    pub fn unchecked_into<T: JsCast>(&self) -> T {
        self.0.clone().unchecked_into::<T>()
    }

    fn shadow_clone(&self) -> Self {
        Self(self.0.clone_node_with_deep(false).expect_throw(
            "render::element::WsElement::clone",
        ).unchecked_into())
    }


    pub fn set_id(&self, id: &str) {
        self.0.set_id(id);
    }

    pub fn set_text_content(&self, text: Option<&str>) {
        self.0.set_text_content(text);
    }

    pub fn set_str_attribute(&self, attribute_name: &str, attribute_value: &str) {
        self.0
            .set_attribute(attribute_name, attribute_value)
            .expect_throw("dom::element::WsElement::set_str_attribute");
    }

    pub fn remove_attribute(&self, attribute_name: &str) {
        self.0.remove_attribute(attribute_name)
            .expect_throw("dom::element::WsElement::remove_attribute");
    }

    pub fn set_attribute<T: AttributeValueAsString>(&self, attribute_name: &str, attribute_value: T) {
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
        self.0
            .class_list()
            .add_1(class_name)
            .expect_throw("dom::element::WsElement::add_class");
    }

    pub fn remove_class(&self, class_name: &str) {
        self.0
            .class_list()
            .remove_1(class_name)
            .expect_throw("dom::element::WsElement::remove_class");
    }


    pub fn scroll_to_view_with_bool(&self, align_to_top: bool) {
        self.0.scroll_into_view_with_bool(align_to_top);
    }

    pub fn scroll_to_view_with_options(&self, options: &web_sys::ScrollIntoViewOptions) {
        self.0.scroll_into_view_with_scroll_into_view_options(options);
    }
}

