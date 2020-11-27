mod attributes;
mod nodes;

static SVG_NAMESPACE: &'static str = "http://www.w3.org/2000/svg";

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
        self.0.nodes()
    }

    pub fn static_nodes(self) -> nodes::SvgStaticNodesOwned<'a, C> {
        self.0.static_nodes()
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    pub fn render(self, value: impl SvgRender<C>) -> nodes::SvgNodesOwned<'a, C> {
        self.0.render(value)
    }

    // pub fn render_ref(
    //     self,
    //     value: &impl SvgRenderRef<C>,
    // ) -> nodes::SvgNodesOwned<'a, C> {
    //     self.0.render_ref(value)
    // }

    pub fn r#static(self, value: impl SvgStaticRender<C>) -> nodes::SvgNodesOwned<'a, C> {
        self.0.r#static(value)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.0
            .list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, mode, tag, render)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        self.0.keyed_list(items, mode)
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.0.component(child)
    }
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

// impl<'a, C: crate::component::Component> From<SvgUpdater<'a, C>> for nodes::NodesOwned<'a, C> {
//     fn from(hu: SvgUpdater<'a, C>) -> Self {
//         Self::from(hu.0)
//     }
// }

// impl<'a, C: crate::component::Component> nodes::DomBuilder<C> for SvgUpdater<'a, C> {
//     type Output = nodes::NodesOwned<'a, C>;
// }
