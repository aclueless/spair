use wasm_bindgen::UnwrapThrowExt;

mod attributes;
mod nodes;
mod non_keyed_list;
mod renderable;

pub use attributes::*;
pub use nodes::*;
pub use renderable::*;

static SVG_NAMESPACE: &'static str = "http://www.w3.org/2000/svg";

impl super::Element {
    pub fn new_svg_element(tag: &str) -> Self {
        Self {
            element_type: "svg".into(),
            ws_element: crate::utils::document()
                .create_element_ns(Some(SVG_NAMESPACE), tag)
                .expect_throw("Unable to create new svg element"),
            attributes: Default::default(),
            nodes: Default::default(),
        }
    }
}

impl crate::dom::nodes::NodeList {
    fn create_new_svg_element(
        &mut self,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let svg = super::Element::new_svg_element(tag);
        svg.insert_before(parent, next_sibling);
        self.0.push(crate::dom::nodes::Node::Element(svg));
    }

    pub fn check_or_create_svg_element_ns(
        &mut self,
        tag: &str,
        index: usize,
        parent_status: super::ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> super::ElementStatus {
        if index == self.0.len() {
            self.create_new_svg_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    // TODO: Need to reduce code duplication of this and the non-svg method
    pub fn check_or_create_svg_element_for_non_keyed_list(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> super::ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            super::ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.create_new_svg_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            let element = self.0[0].clone();
            match &element {
                crate::dom::nodes::Node::Element(element) => {
                    element.insert_before(parent, next_sibling)
                }
                _ => panic!("non-keyed-list svg: internal bug?"),
            }
            self.0.push(element);
            super::ElementStatus::JustCloned
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

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

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

    pub fn render(self, value: impl super::SvgRender<C>) -> SvgNodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    pub fn r#static(self, value: impl super::SvgStaticRender<C>) -> SvgNodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let static_nodes = nodes_owned.static_nodes_ref();
        value.render(static_nodes);
        nodes_owned
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, SvgUpdater<'c, C>),
    {
        let parent = self.element.ws_element.as_ref();
        let use_template = mode.use_template();

        let mut non_keyed_list_updater = non_keyed_list::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut self.element.nodes,
            tag,
            parent,
            None,
            use_template,
        );
        non_keyed_list_updater.update(items, render);
    }
}
