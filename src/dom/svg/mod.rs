use wasm_bindgen::UnwrapThrowExt;

mod attributes;
mod nodes;

use attributes::*;
use nodes::*;

impl super::Element {
    pub fn new_svg() -> Self {
        Self {
            element_type: "svg".into(),
            ws_element: crate::utils::document()
                .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
                .expect_throw("Unable to create new svg element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }
}

pub struct SvgUpdater<'a, C> {
    pub(super) comp: &'a crate::component::Comp<C>,
    pub(super) state: &'a C,

    pub(super) index: usize,
    pub(super) status: super::ElementStatus,
    pub(super) element: &'a mut super::Element,
}

impl<'a, C: crate::component::Component> SvgUpdater<'a, C> {
    pub(crate) fn new(
        comp: &'a crate::component::Comp<C>,
        state: &'a C,
        element: &'a mut super::Element,
        status: super::ElementStatus,
    ) -> Self {
        Self {
            comp,
            state,
            index: 0,
            status,
            element,
        }
    }

    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.comp.clone()
    }

    pub fn ws_element(&self) -> web_sys::Element {
        self.element.ws_element.clone()
    }

    pub fn clear(self) {
        let parent = self.element.ws_element.as_ref();
        self.element.nodes.clear(parent);
    }

    pub fn static_attributes(self) -> SvgStaticAttributes<'a, C> {
        SvgStaticAttributes::new(self)
    }

    pub fn nodes(self) -> SvgNodesOwned<'a, C> {
        SvgNodesOwned::from_svg_updater(self)
    }

    pub fn static_nodes(self) -> SvgStaticNodesOwned<'a, C> {
        SvgStaticNodesOwned::from_svg_updater(self)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}
}
