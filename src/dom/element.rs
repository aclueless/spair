use super::{AttributeValueList, ElementType, Nodes, ParentAndChild};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

#[derive(Debug)]
pub struct Element {
    element_type: ElementType,
    ws_element: web_sys::Element,
    attributes: AttributeValueList,
    nodes: Nodes,
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let ws_element = self.ws_element.clone_node_with_deep(false).expect_throw(
            "render::element::Element::clone::self.ws_element.clone_node_with_deep(false)",
        );
        let nodes = self.nodes.clone();
        nodes.append_to(&ws_element);

        Self {
            element_type: self.element_type,
            ws_element: ws_element.unchecked_into(),
            nodes,
            attributes: self.attributes.clone(),
        }
    }
}

impl ParentAndChild for Element {
    fn ws_node(&self) -> &web_sys::Node {
        self.ws_element.as_ref()
    }
}

impl Element {
    pub fn new_ns(ns: Option<&str>, tag: &str) -> Self {
        let document = crate::utils::document();
        Self {
            element_type: tag.into(),
            ws_element: if ns.is_some() {
                document.create_element_ns(ns, tag).expect_throw(
                    "render::element::Element::new_ns::document.create_element_ns(ns, tag)",
                )
            } else {
                document
                    .create_element(tag)
                    .expect_throw("render::element::Element::new_ns::document.create_element(tag)")
            },
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    pub fn from_ws_element(ws_element: web_sys::Element) -> Self {
        Self {
            element_type: ws_element.tag_name().to_ascii_lowercase().as_str().into(),
            ws_element,
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }

    // This is intended to use with child component
    pub fn replace_ws_element(&mut self, ws_element: web_sys::Element) {
        self.ws_element = ws_element;
        self.nodes.append_to(self.ws_element.as_ref());
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.count() == 0
    }

    pub fn ws_element(&self) -> &web_sys::Element {
        &self.ws_element
    }

    pub fn ws_node_and_nodes_mut(&mut self) -> (&web_sys::Node, &mut Nodes) {
        (self.ws_element.as_ref(), &mut self.nodes)
    }

    pub fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.ws_element.unchecked_ref()
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

    pub fn scroll_to_view_with_bool(&self, align_to_top: bool) {
        self.ws_element.scroll_into_view_with_bool(align_to_top);
    }

    pub fn scroll_to_view_with_options(&self, options: &web_sys::ScrollIntoViewOptions) {
        self.ws_element
            .scroll_into_view_with_scroll_into_view_options(options);
    }

    pub fn set_bool_attribute(&mut self, name: &str, value: bool) {
        if value {
            self.ws_element()
                .set_attribute(name, "")
                .expect_throw("render::element::Element::set_bool_attribute::set");
        } else {
            self.ws_element()
                .remove_attribute(name)
                .expect_throw("render::element::Element::set_bool_attribute::remove");
        }
    }

    pub fn set_str_attribute(&mut self, name: &str, value: &str) {
        self.ws_element()
            .set_attribute(name, value)
            .expect_throw("render::element::Element::set_str_attribute::set");
    }

    pub fn set_i32_attribute(&mut self, name: &str, value: i32) {
        self.ws_element()
            .set_attribute(name, &value.to_string())
            .expect_throw("render::element::Element::set_i32_attribute::set");
    }

    pub fn set_u32_attribute(&mut self, name: &str, value: u32) {
        self.ws_element()
            .set_attribute(name, &value.to_string())
            .expect_throw("render::element::Element::set_u32_attribute::set");
    }

    pub fn set_f64_attribute(&mut self, name: &str, value: f64) {
        self.ws_element()
            .set_attribute(name, &value.to_string())
            .expect_throw("render::element::Element::set_f64_attribute::set");
    }
}
