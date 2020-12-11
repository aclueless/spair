macro_rules! create_methods_for_tags {
    ($($tag:ident)+) => {
        $(
            fn $tag(self, f: impl FnOnce(super::HtmlUpdater<C>)) -> Self::Output {
                self.render_element(stringify!($tag), f)
            }
        )+
    }
}

pub trait DomBuilder<C: crate::component::Component>: Sized {
    type Output: From<Self> + crate::dom::nodes::DomBuilder<C>;

    fn match_if(self, f: impl FnOnce(crate::dom::nodes::MatchIfUpdater<C>)) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        f(this.get_match_if_and_increase_index());
        this
    }

    fn render_element(self, tag: &str, f: impl FnOnce(super::HtmlUpdater<C>)) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            f(this.get_element_and_increase_index(tag).into());
        } else {
            this.next_index();
        }
        this
    }

    create_methods_for_tags! {
        a abbr address area article aside audio
        b bdi bdo blockquote button br
        canvas caption cite code col colgroup
        data datalist dd del details dfn dialog div dl dt
        em embed
        fieldset figcaption figure footer form
        h1 h2 h3 h4 h5 h6 header hgroup hr
        i iframe img input ins
        kbd
        label legend li
        main map mark menu meter
        nav
        object ol optgroup option output
        p param picture pre progress
        q
        rp rt ruby
        s samp section select slot small source span strong sub summary sup
        table tbody td template textarea tfoot th thead time tr track
        u ul
        var video
        wbr //should be specialized?
    }

    fn line_break(self) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            this.get_element_and_increase_index("br");
        } else {
            this.next_index();
        }
        this
    }

    fn horizontal_line(self) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            this.get_element_and_increase_index("hr");
        } else {
            this.next_index();
        }
        this
    }

    fn raw_wrapper(self, raw_wrapper: &impl crate::dom::RawWrapper<C>) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.just_created() {
            let ws_element = raw_wrapper.ws_element();
            // TODO: should raw element stores in its own variant?
            //      This store the ws_element of the RawWrapper as a crate::dom::Element,
            //      it may cause a problem when the RawWrapper in side a list element
            let element = crate::dom::Element::from_ws_element(ws_element.clone());
            this.store_raw_wrapper(element);
            raw_wrapper.mounted();
        }
        this.next_index();

        this
    }
}

pub struct HtmlNodeListUpdater<'a, C> {
    u: crate::dom::nodes::NodeListUpdater<'a, C>,
    #[cfg(feature = "partial-non-keyed-list")]
    select_element_value: crate::dom::SelectElementValue,
}

#[cfg(feature = "partial-non-keyed-list")]
impl<'a, C> Drop for HtmlNodeListUpdater<'a, C> {
    fn drop(&mut self) {
        self.select_element_value
            .set_select_element_value(self.u.parent());
    }
}

impl<'a, C> From<super::HtmlUpdater<'a, C>> for HtmlNodeListUpdater<'a, C> {
    fn from(u: super::HtmlUpdater<'a, C>) -> Self {
        Self {
            u: From::from(u.u),
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: u.select_element_value,
        }
    }
}

impl<'a, C> From<crate::dom::nodes::NodeListUpdater<'a, C>> for HtmlNodeListUpdater<'a, C> {
    fn from(u: crate::dom::nodes::NodeListUpdater<'a, C>) -> Self {
        Self {
            u,
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: crate::dom::SelectElementValue::none(),
        }
    }
}

pub struct StaticNodesOwned<'a, C>(HtmlNodeListUpdater<'a, C>);
pub struct NodesOwned<'a, C>(HtmlNodeListUpdater<'a, C>);
pub struct StaticNodes<'n, 'h: 'n, C>(&'n mut HtmlNodeListUpdater<'h, C>);
pub struct Nodes<'n, 'h: 'n, C>(&'n mut HtmlNodeListUpdater<'h, C>);

impl<'a, C> From<super::HtmlUpdater<'a, C>> for StaticNodesOwned<'a, C> {
    fn from(u: super::HtmlUpdater<'a, C>) -> Self {
        Self(u.into())
    }
}

impl<'a, C> From<super::HtmlUpdater<'a, C>> for NodesOwned<'a, C> {
    fn from(u: super::HtmlUpdater<'a, C>) -> Self {
        Self(u.into())
    }
}

impl<'a, C> From<crate::dom::nodes::NodeListUpdater<'a, C>> for NodesOwned<'a, C> {
    fn from(u: crate::dom::nodes::NodeListUpdater<'a, C>) -> Self {
        Self(u.into())
    }
}

impl<'a, C: crate::component::Component> From<crate::dom::StaticAttributes<'a, C>>
    for NodesOwned<'a, C>
{
    fn from(sa: crate::dom::StaticAttributes<'a, C>) -> Self {
        sa.nodes()
    }
}

impl<'a, C: crate::component::Component> StaticNodesOwned<'a, C> {
    /// Use this method when you are done with your object. It is useful in single-line closures
    /// where you don't want to add a semicolon `;` but the compiler complains that "expected `()`
    /// but found `something-else`"
    pub fn done(self) {}

    pub fn state(&self) -> &'a C {
        self.0.u.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.u.comp()
    }

    #[cfg(feature = "svg")]
    pub fn svg(mut self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> StaticNodesOwned<'a, C> {
        // where are in static mode
        if self.0.u.parent_status() == crate::dom::ElementStatus::JustCreated {
            self.0.u.svg(f);
        } else {
            self.0.u.next_index();
        }
        self
    }

    pub fn nodes(self) -> NodesOwned<'a, C> {
        NodesOwned(self.0)
    }

    pub fn render(mut self, value: impl crate::dom::Render<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(mut self, value: &impl crate::dom::RenderRef<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(mut self, value: impl crate::dom::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::dom::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.u.parent_status() != crate::dom::ElementStatus::Existing {
    //         value.render(self.nodes()).static_nodes()
    //     } else {
    //         self.0.u.next_index();
    //         self
    //     }
    // }
}

impl<'a, C: crate::component::Component> NodesOwned<'a, C> {
    pub(in crate::dom) fn nodes_ref<'n>(&'n mut self) -> Nodes<'n, 'a, C> {
        Nodes(&mut self.0)
    }

    pub(in crate::dom) fn static_nodes_ref<'n>(&'n mut self) -> StaticNodes<'n, 'a, C> {
        StaticNodes(&mut self.0)
    }

    /// Use this method when you are done with your object. It is useful in single-line closures
    /// where you don't want to add a semicolon `;` but the compiler complains that "expected `()`
    /// but found `something-else`"
    pub fn done(self) {}

    pub fn state(&self) -> &'a C {
        self.0.u.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.u.comp()
    }

    #[cfg(feature = "svg")]
    pub fn svg(mut self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> Self {
        self.0.u.svg(f);
        self
    }

    pub fn static_nodes(self) -> StaticNodesOwned<'a, C> {
        StaticNodesOwned(self.0)
    }

    pub fn render(mut self, value: impl crate::dom::Render<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(mut self, value: &impl crate::dom::RenderRef<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(mut self, value: impl crate::dom::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::dom::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.u.parent_status() != crate::dom::ElementStatus::Existing {
    //         value.render(self)
    //     } else {
    //         self.0.u.next_index();
    //         self
    //     }
    // }

    pub(crate) fn update_text(mut self, text: &str) -> Self {
        self.0.u.update_text(text);
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
        tag: &str,
        render: R,
    ) -> Self
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.0.u.list_with_render(items, mode, tag, render);
        self
    }
}

impl<'n, 'h, C: crate::component::Component> StaticNodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.u.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.u.comp()
    }

    #[cfg(feature = "svg")]
    pub fn svg(self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> Self {
        // where are in static mode
        if self.0.u.parent_status() == crate::dom::ElementStatus::JustCreated {
            self.0.u.svg(f);
        } else {
            self.0.u.next_index();
        }
        self
    }

    pub fn nodes(self) -> Nodes<'n, 'h, C> {
        Nodes(self.0)
    }

    pub fn render(self, value: impl crate::dom::Render<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(self, value: &impl crate::dom::RenderRef<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(self, value: impl crate::dom::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    pub(crate) fn static_text(self, text: &str) -> Self {
        self.0.u.static_text(text);
        self
    }
}

impl<'n, 'h, C: crate::component::Component> Nodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.u.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.u.comp()
    }

    #[cfg(feature = "svg")]
    pub fn svg(self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> Self {
        self.0.u.svg(f);
        self
    }

    pub fn static_nodes(self) -> StaticNodes<'n, 'h, C> {
        StaticNodes(self.0)
    }

    pub fn render(self, value: impl crate::dom::Render<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(self, value: &impl crate::dom::RenderRef<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(self, value: impl crate::dom::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    pub(crate) fn update_text(self, text: &str) -> Self {
        self.0.u.update_text(text);
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
        tag: &str,
        render: R,
    ) -> Self
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.0.u.list_with_render(items, mode, tag, render);
        self
    }
}

impl<'a, C: crate::component::Component> crate::dom::nodes::DomBuilder<C>
    for StaticNodesOwned<'a, C>
{
    fn require_render(&self) -> bool {
        self.0.u.parent_status() == crate::dom::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.u.just_created()
    }

    fn next_index(&mut self) {
        self.0.u.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.u.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.u.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.u.store_raw_wrapper(element);
    }
}

impl<'a, C: crate::component::Component> crate::dom::nodes::DomBuilder<C> for NodesOwned<'a, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.u.just_created()
    }

    fn next_index(&mut self) {
        self.0.u.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.u.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.u.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.u.store_raw_wrapper(element);
    }
}

impl<'n, 'h, C: crate::component::Component> crate::dom::nodes::DomBuilder<C>
    for StaticNodes<'n, 'h, C>
{
    fn require_render(&self) -> bool {
        self.0.u.parent_status() == crate::dom::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.u.just_created()
    }

    fn next_index(&mut self) {
        self.0.u.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.u.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.u.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.u.store_raw_wrapper(element);
    }
}

impl<'n, 'h, C: crate::component::Component> crate::dom::nodes::DomBuilder<C> for Nodes<'n, 'h, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.u.just_created()
    }

    fn next_index(&mut self) {
        self.0.u.next_index();
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.u.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.u.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.u.store_raw_wrapper(element);
    }
}

impl<'a, C: crate::component::Component> DomBuilder<C> for StaticNodesOwned<'a, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> DomBuilder<C> for NodesOwned<'a, C> {
    type Output = Self;
}

impl<'n, 'h, C: crate::component::Component> DomBuilder<C> for StaticNodes<'n, 'h, C> {
    type Output = Self;
}

impl<'n, 'h, C: crate::component::Component> DomBuilder<C> for Nodes<'n, 'h, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> DomBuilder<C> for crate::dom::StaticAttributes<'a, C> {
    type Output = NodesOwned<'a, C>;
}
