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
