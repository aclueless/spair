pub mod attributes;
pub mod nodes;
pub mod renderable;

/// This struct provide methods for setting properties/attributes and adding child nodes for
/// HTML elements.
pub struct HtmlUpdater<'a, C> {
    pub(super) u: crate::dom::element::ElementUpdater<'a, C>,
    select_element_value: crate::dom::SelectElementValue,
}

impl<'a, C> From<super::ElementUpdater<'a, C>> for HtmlUpdater<'a, C> {
    fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self {
        Self {
            u,
            select_element_value: crate::dom::SelectElementValue::none(),
        }
    }
}

impl<'a, C: crate::component::Component> HtmlUpdater<'a, C> {
    pub(crate) fn ws_element_clone(&self) -> web_sys::Element {
        self.u.ws_element().clone()
    }

    pub(super) fn status(&self) -> super::ElementStatus {
        self.u.status()
    }

    pub(super) fn select_element_value_mut(&mut self) -> &mut crate::dom::SelectElementValue {
        &mut self.select_element_value
    }

    /// Use this method when you are done with your object. It is useful in single-line closures
    /// where you don't want to add a semicolon `;` but the compiler complains that "expected `()`
    /// but found `something-else`"
    pub fn done(self) {}

    pub fn state(&self) -> &'a C {
        self.u.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.u.comp()
    }

    pub fn static_attributes(self) -> attributes::StaticAttributes<'a, C> {
        attributes::StaticAttributes::from(self)
    }

    pub fn nodes(self) -> super::NodesOwned<'a, C> {
        nodes::NodesOwned::from(self)
    }

    pub fn static_nodes(self) -> super::StaticNodesOwned<'a, C> {
        nodes::StaticNodesOwned::from(self)
    }

    pub fn render(self, value: impl super::Render<C>) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    pub fn render_ref(self, value: &impl super::RenderRef<C>) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let nodes = nodes_owned.nodes_ref();
        value.render(nodes);
        nodes_owned
    }

    pub fn r#static(self, value: impl super::StaticRender<C>) -> super::NodesOwned<'a, C> {
        let mut nodes_owned = self.nodes();
        let static_nodes = nodes_owned.static_nodes_ref();
        value.render(static_nodes);
        nodes_owned
    }

    pub fn update_text(self, text: &str) -> super::NodesOwned<'a, C> {
        let nodes_owned = self.nodes();
        nodes_owned.update_text(text)
    }

    #[cfg(feature = "svg")]
    pub fn svg(self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> super::NodesOwned<'a, C> {
        let nodes = self.nodes();
        nodes.svg(f)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: renderable::ListItem<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
    }

    pub fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &'a str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        let _must_set_select_element_value_after_this =
            self.u.list_with_render(items, mode, tag, render);

        //The hack start in AttributeSetter::value
        self.select_element_value
            .set_select_element_value(self.u.ws_element().as_ref());
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
    ) where
        for<'k> I: super::Keyed<'k> + super::ListItem<C>,
    {
        self.u.keyed_list(items, mode);
        //The hack start in AttributeSetter::value
        self.select_element_value
            .set_select_element_value(self.u.ws_element().as_ref());
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list_with_render<I, G, K, R, U>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        mode: super::ListElementCreation,
        tag: &'a str,
        get_key: G,
        render: R,
    ) where
        for<'i> G: Fn(&'i I) -> K,
        K: 'a + Into<super::Key> + PartialEq<super::Key>,
        for<'i> R: Fn(&'i I, U),
        for<'c> U: From<crate::dom::ElementUpdater<'c, C>>,
    {
        let _must_set_select_element_value_after_this = self
            .u
            .keyed_list_with_render(items, mode, tag, get_key, render);

        //The hack start in AttributeSetter::value
        self.select_element_value
            .set_select_element_value(self.u.ws_element().as_ref());
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.u.component(child)
    }
}

impl<'a, C: crate::component::Component> crate::dom::attributes::AttributeSetter
    for HtmlUpdater<'a, C>
{
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.u.ws_html_element()
    }

    fn ws_element(&self) -> &web_sys::Element {
        self.u.ws_element()
    }

    fn element_type(&self) -> crate::dom::ElementType {
        self.u.element_type()
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.u.store_listener(listener);
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        self.u.check_bool_attribute(value)
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        self.u.check_str_attribute(value)
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        self.u.check_i32_attribute(value)
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        self.u.check_u32_attribute(value)
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        self.u.check_f64_attribute(value)
    }
}

impl<'a, C: crate::component::Component> attributes::AttributeValueSetter for HtmlUpdater<'a, C> {
    fn set_selected_value(&mut self, value: Option<&str>) {
        //self.u.set_selected_value(value)
        self.select_element_value.set_selected_value(value);
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        //self.u.set_selected_index(index)
        self.select_element_value.set_selected_index(index);
    }
}
impl<'a, C: crate::component::Component> attributes::AttributeSetter<C> for HtmlUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> crate::dom::attributes::EventSetter for HtmlUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> nodes::DomBuilder<C> for HtmlUpdater<'a, C> {
    type Output = nodes::NodesOwned<'a, C>;
}
