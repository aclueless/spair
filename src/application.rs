use wasm_bindgen::UnwrapThrowExt;

pub fn start<C: crate::component::Component>(state: C, id: &str) {
    let root = crate::utils::document()
        .get_element_by_id(id)
        .expect_throw("No element associated with the specified id (to use as a root element)");
    let rc_comp = crate::component::RcComp::with_state_and_element(state, Some(root));
    rc_comp.first_render();

    // It is the main component of the app, hence it is reasonable to just forget it.
    std::mem::forget(rc_comp);
}

pub trait Application: crate::component::Component {
    fn mount_to_element(self, root: web_sys::Element) {
        let rc_comp = crate::component::RcComp::with_state_and_element(self, Some(root));
        rc_comp.first_render();

        // It is the main component of the app, hence it is reasonable to just forget it.
        std::mem::forget(rc_comp);
    }

    fn mount_to(self, id: &str) {
        let root = crate::utils::document()
            .get_element_by_id(id)
            .expect_throw("No element associated with the specified id (to use as a root element)");
        self.mount_to_element(root);
    }

    fn mount_to_body(self) {
        let root = crate::utils::document()
            .body()
            .expect("document body")
            .into();
        self.mount_to_element(root);
    }
}

impl<C: crate::component::Component> Application for C {}
