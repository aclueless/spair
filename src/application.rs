use wasm_bindgen::UnwrapThrowExt;

pub trait Application: crate::component::Component {
    fn with_comp(comp: crate::component::Comp<Self>) -> Self;

    fn mount_to_element(root: web_sys::Element) {
        root.set_text_content(None);
        let rc_comp = crate::component::RcComp::new(Some(root));
        let state = Self::with_comp(rc_comp.comp());
        rc_comp.set_state(state);
        rc_comp.first_render();

        // It is the main component of the app, hence it is reasonable to just forget it.
        std::mem::forget(rc_comp);
    }

    fn mount_to(id: &str) {
        let root = crate::utils::document()
            .get_element_by_id(id)
            .expect_throw("No element associated with the specified id (to use as a root element)");
        Self::mount_to_element(root);
    }

    fn mount_to_body() {
        let root = crate::utils::document()
            .body()
            .expect("document body")
            .into();
        Self::mount_to_element(root);
    }
}
