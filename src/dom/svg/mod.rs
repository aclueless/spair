use wasm_bindgen::UnwrapThrowExt;

impl super::Element {
    pub fn new_svg(tag: &str) -> Self {
        Self {
            element_type: tag.into(),
            ws_element: crate::utils::document()
                .create_element_ns(Some("http://www.w3.org/2000/svg"), tag)
                .expect_throw("Unable to create new svg element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }
}
