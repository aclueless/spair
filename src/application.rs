use crate::{
    component::{Comp, Component, RcComp},
    routing::{self, Routes},
};
use wasm_bindgen::UnwrapThrowExt;

pub trait Application: Component {
    fn init(comp: &Comp<Self>) -> Self;

    /// If your Component::Routes is not `()`, you must override this to provide the actual router instance
    fn init_router(_comp: &Comp<Self>) -> Option<<<Self as Component>::Routes as Routes>::Router> {
        None
    }

    fn mount_to_element(root: web_sys::Element) {
        root.set_text_content(None);
        let rc_comp = RcComp::with_root(Some(root));
        let comp = rc_comp.comp();

        // Must set the router before initing the state.
        match Self::init_router(&comp) {
            Some(router) => routing::set_router(router),
            None if std::any::TypeId::of::<()>()
                != std::any::TypeId::of::<<<Self as Component>::Routes as Routes>::Router>() =>
            {
                log::warn!(
                    "You may want to implement `Application::init_router()` to return Some(router)"
                );
            }
            _ => {}
        }

        // In case that the root-component (A) have a child-component (C) that being construct by A
        // in A's Application::init(). Currently, C, will immediately register its callback to the
        // router, hence, the router must be already set before initing A's state.
        let state = Application::init(&comp);
        rc_comp.set_state(state);

        routing::execute_routing::<<<Self as Component>::Routes as Routes>::Router>();

        rc_comp.first_render();
        // It is the root component of the app, hence it is reasonable to just forget it.
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
