use wasm_bindgen::UnwrapThrowExt;

pub mod attributes;
pub mod nodes;

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

pub trait SvgRender<C: crate::component::Component> {
    fn render(self, nodes: nodes::SvgNodes<C>);
}

pub trait SvgStaticRender<C: crate::component::Component> {
    fn render(self, nodes: nodes::SvgStaticNodes<C>);
}

macro_rules! impl_render_with_to_string {
    ($($type:ident)+) => {
        $(
            impl<C: crate::component::Component> SvgRender<C> for $type {
                fn render(self, nodes: nodes::SvgNodes<C>) {
                    nodes.update_text(&self.to_string());
                }
            }

            impl<C: crate::component::Component> SvgStaticRender<C> for $type {
                fn render(self, nodes: nodes::SvgStaticNodes<C>) {
                    nodes.static_text(&self.to_string());
                }
            }
        )+
    }
}

impl_render_with_to_string! {
    i8 i16 i32 i64 u8 u16 u32 u64 isize usize f32 f64 bool char
}

impl<C: crate::component::Component> SvgRender<C> for &str {
    fn render(self, nodes: nodes::SvgNodes<C>) {
        nodes.update_text(self);
    }
}

impl<C: crate::component::Component> SvgStaticRender<C> for &str {
    fn render(self, nodes: nodes::SvgStaticNodes<C>) {
        nodes.static_text(self);
    }
}

/// This struct provide methods for setting properties/attributes and adding child nodes for
/// HTML elements.
pub struct SvgUpdater<'a, C>(crate::dom::element::ElementUpdater<'a, C>);

impl<'a, C> From<super::ElementUpdater<'a, C>> for SvgUpdater<'a, C> {
    fn from(eu: crate::dom::ElementUpdater<'a, C>) -> Self {
        Self(eu)
    }
}

impl<'a, C: crate::component::Component> SvgUpdater<'a, C> {
    pub(crate) fn ws_element_clone(&self) -> web_sys::Element {
        self.0.ws_element().clone()
    }

    /// Use this method when you are done with your object. It is useful in single-line closures
    /// where you don't want to add a semicolon `;` but the compiler complains that "expected `()`
    /// but found `something-else`"
    pub fn done(self) {}

    pub fn state(&self) -> &'a C {
        self.0.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp()
    }

    pub fn static_attributes(self) -> attributes::SvgStaticAttributes<'a, C> {
        From::from(self.0)
    }

    pub fn nodes(self) -> nodes::SvgNodesOwned<'a, C> {
        self.0.svg_nodes()
    }

    pub fn static_nodes(self) -> nodes::SvgStaticNodesOwned<'a, C> {
        self.0.svg_static_nodes()
    }

    pub fn render(self, value: impl SvgRender<C>) -> nodes::SvgNodesOwned<'a, C> {
        self.0.svg_render(value)
    }

    // pub fn render_ref(
    //     self,
    //     value: &impl SvgRenderRef<C>,
    // ) -> nodes::SvgNodesOwned<'a, C> {
    //     self.0.render_ref(value)
    // }

    pub fn r#static(self, value: impl SvgStaticRender<C>) -> nodes::SvgNodesOwned<'a, C> {
        self.0.svg_static(value)
    }

    // pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    // where
    //     I: crate::renderable::ListItem<C>,
    // {
    //     self.0
    //         .list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
    // }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, SvgUpdater<'c, C>),
    {
        self.0.svg_list_with_render(items, mode, tag, render)
    }

    // #[cfg(feature = "keyed-list")]
    // pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    // where
    //     for<'k> I: super::KeyedListItem<'k, C>,
    // {
    //     self.0.keyed_list(items, mode)
    // }

    // pub fn component<CC: crate::component::Component>(
    //     self,
    //     child: &crate::component::ChildComp<CC>,
    // ) {
    //     self.0.component(child)
    // }
}

impl<'a, C: crate::component::Component> crate::dom::attributes::AttributeSetter
    for SvgUpdater<'a, C>
{
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.ws_html_element()
    }

    fn ws_element(&self) -> &web_sys::Element {
        self.0.ws_element()
    }

    fn element_type(&self) -> crate::dom::ElementType {
        self.0.element_type()
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0.store_listener(listener);
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        self.0.check_bool_attribute(value)
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        self.0.check_str_attribute(value)
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        self.0.check_i32_attribute(value)
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        self.0.check_u32_attribute(value)
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        self.0.check_f64_attribute(value)
    }

    fn set_selected_value(&mut self, value: Option<&str>) {
        self.0.set_selected_value(value)
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.0.set_selected_index(index)
    }
}

impl<'a, C: crate::component::Component> attributes::SvgAttributeSetter<C> for SvgUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> crate::dom::attributes::EventSetter for SvgUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> From<SvgUpdater<'a, C>> for nodes::SvgNodesOwned<'a, C> {
    fn from(su: SvgUpdater<'a, C>) -> Self {
        Self::from(su.0)
    }
}

impl<'a, C: crate::component::Component> nodes::SvgBuilder<C> for SvgUpdater<'a, C> {
    type Output = nodes::SvgNodesOwned<'a, C>;
}
