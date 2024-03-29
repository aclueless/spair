use std::any::TypeId;

use wasm_bindgen::UnwrapThrowExt;

use super::{
    AttributesOnly, HtmlElementUpdater, HtmlTag, SelectElementValueManager, StaticAttributes,
    StaticAttributesOnly,
};
#[cfg(feature = "svg")]
use crate::render::svg::{SvgElementUpdater, SvgTag};
use crate::{
    component::{Child, Comp, Component},
    dom::ComponentRef,
    render::base::{ElementUpdaterMut, MatchIfUpdater, NodesUpdater, NodesUpdaterMut, TextRender},
};

#[cfg(feature = "queue-render")]
use crate::queue_render::val::QrVal;

pub struct HtmlNodesUpdater<'n, C: Component> {
    nodes_updater: NodesUpdater<'n, C>,
    // Just keep this value until the completion of the build of the whole node list
    // After done building the node list, this value will be dropped. The Drop::drop method
    // will execute setting value for the <select> element
    _select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'n, C: Component> HtmlNodesUpdater<'n, C> {
    pub(crate) fn new(nodes_updater: NodesUpdater<'n, C>) -> Self {
        Self {
            nodes_updater,
            _select_element_value_manager: None,
        }
    }
}

impl<'n, C: Component> NodesUpdaterMut<'n, C> for HtmlNodesUpdater<'n, C> {
    fn nodes_updater_mut<'a>(self: &'a mut HtmlNodesUpdater<'n, C>) -> &'a mut NodesUpdater<'n, C> {
        &mut self.nodes_updater
    }
}

pub trait HemsHandMade<'n, C: Component>: Sized {
    type Output: From<Self> + NodesUpdaterMut<'n, C>;

    fn line_break(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        render.get_element_updater(HtmlTag("br"));
        this
    }

    fn horizontal_line(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        render.get_element_updater(HtmlTag("hr"));
        this
    }

    fn match_if(self, f: impl FnOnce(HtmlMatchIfUpdater<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        let mi = render.get_match_if_updater();
        let mi = HtmlMatchIfUpdater(mi);
        f(mi);
        this
    }

    /// A convenience method to render an Option<T> with a text fallback for None.
    /// If you want to render something else for None, then use `.match_if()`.
    fn render_option<T>(
        self,
        value: Option<T>,
        fn_for_some: impl FnOnce(T, Nodes<C>),
        text_for_none: &'static str,
    ) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        let mi = render.get_match_if_updater();
        let mi = HtmlMatchIfUpdater(mi);
        match value {
            None => mi
                .render_on_arm_index(std::any::TypeId::of::<isize>())
                .update_text(text_for_none)
                .done(),
            Some(value) => mi
                .render_on_arm_index(std::any::TypeId::of::<usize>())
                .rfn(|nodes| fn_for_some(value, nodes))
                .done(),
        }
        this
    }

    #[cfg(feature = "queue-render")]
    fn qr_match_if<T: 'static>(
        self,
        value: &QrVal<T>,
        f: impl Fn(&T, HtmlMatchIfUpdater<C>) + 'static,
    ) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if let Some(mi) = render.create_qr_match_if(move |t, mi| {
            let mi = HtmlMatchIfUpdater(mi);
            f(t, mi);
        }) {
            value
                .content()
                .try_borrow_mut()
                .expect_throw("render::html::nodes::HemsHandMade::qr_match_if")
                .add_render(Box::new(mi));
        }
        this
    }

    #[cfg(feature = "svg")]
    fn svg(self, element_updater: impl FnOnce(SvgElementUpdater<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if let Some(e) = render.get_element_updater(SvgTag("svg")) {
            element_updater(e);
        }
        this
    }

    fn component_ref(self, cr: Option<Box<dyn ComponentRef>>) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if let Some(cr) = cr {
            render.component_ref(cr);
        }
        render.next_index();
        this
    }

    fn component_owned<CC: Component, T: 'static + Clone + PartialEq>(
        self,
        create_child_comp: impl FnOnce(&C, &Comp<C>) -> Child<C, CC, T>,
    ) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            render.component_owned(create_child_comp);
        }
        render.next_index();
        this
    }

    /// Always update the given value (if change), even under `.static_nodes()`.
    /// But be aware that if this is inside a static element (the parent element),
    /// this will only create a text node on creation but never update, because
    /// the parent element is static.
    fn update_text(self, text: impl TextRender<C>) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        text.render(render, true);
        this
    }

    /// Create a text node on the first render, but never update it.
    /// Even under `.update_nodes()`. When you pass an QrVal to this method,
    /// it will always update.
    fn static_text(self, text: impl TextRender<C>) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        text.render(render, false);
        this
    }
}

pub trait UpdateHtmlElement<'n, C, O>: Sized
where
    C: Component,
    O: From<Self> + NodesUpdaterMut<'n, C>,
{
    fn render_element(
        self,
        tag: &'static str,
        element_updater: impl FnOnce(HtmlElementUpdater<C>),
    ) -> O {
        let mut this: O = self.into();
        let render = this.nodes_updater_mut();
        if let Some(e) = render.get_element_updater(HtmlTag(tag)) {
            element_updater(e);
        }
        this
    }
}

#[cfg(test)]
use crate::render::html::TestHtmlMethods;

make_trait_for_element_methods! {
    TestStructs: (TestHtmlMethods)
    TraitName: HemsForDistinctNames
    UpdateElementTraitName: UpdateHtmlElement
    ElementUpdaterType: HtmlElementUpdater
    elements:
        a

        // moved to ../attributes_elements_with_ambiguous_names
        // abbr

        address area article aside audio
        b bdi bdo blockquote button br
        canvas caption

        // moved to ../attributes_elements_with_ambiguous_names
        // cite

        code col colgroup

        // moved to ../attributes_elements_with_ambiguous_names
        // data

        datalist dd del details dfn dialog div dl dt
        em embed
        fieldset figcaption figure footer

        // moved to ../attributes_elements_with_ambiguous_names
        // form

        h1 h2 h3 h4 h5 h6 header hgroup hr
        i iframe img input ins
        kbd

        // moved to ../attributes_elements_with_ambiguous_names
        // label

        legend li
        main map mark menu meter
        nav
        object ol optgroup option output
        p param picture pre progress
        q
        rp rt ruby
        s samp section select slot small source

        // moved to ../attributes_elements_with_ambiguous_names
        // span

        strong sub summary sup
        table tbody td template textarea tfoot th thead time tr track
        u ul
        var video
        wbr //should be specialized?
}

pub struct NodesOwned<'n, C: Component>(HtmlNodesUpdater<'n, C>);
pub struct StaticNodesOwned<'n, C: Component>(HtmlNodesUpdater<'n, C>);
pub struct Nodes<'h, 'n: 'h, C: Component>(&'h mut HtmlNodesUpdater<'n, C>);
pub struct StaticNodes<'h, 'n: 'h, C: Component>(&'h mut HtmlNodesUpdater<'n, C>);

impl<'n, C: Component> NodesOwned<'n, C> {
    fn new(mut r: HtmlNodesUpdater<'n, C>) -> Self {
        r.nodes_updater.set_update_mode();
        Self(r)
    }
}

impl<'n, C: Component> StaticNodesOwned<'n, C> {
    fn new(mut r: HtmlNodesUpdater<'n, C>) -> Self {
        r.nodes_updater.set_static_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> Nodes<'h, 'n, C> {
    pub(crate) fn new(r: &'h mut HtmlNodesUpdater<'n, C>) -> Self {
        r.nodes_updater.set_update_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> StaticNodes<'h, 'n, C> {
    fn new(r: &'h mut HtmlNodesUpdater<'n, C>) -> Self {
        r.nodes_updater.set_static_mode();
        Self(r)
    }
}

impl<'n, C: Component> NodesUpdaterMut<'n, C> for NodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &mut NodesUpdater<'n, C> {
        &mut self.0.nodes_updater
    }
}

impl<'n, C: Component> NodesUpdaterMut<'n, C> for StaticNodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &mut NodesUpdater<'n, C> {
        &mut self.0.nodes_updater
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<'n, C> for Nodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &mut NodesUpdater<'n, C> {
        &mut self.0.nodes_updater
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<'n, C> for StaticNodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &mut NodesUpdater<'n, C> {
        &mut self.0.nodes_updater
    }
}

impl<'n, C: Component> From<NodesOwned<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(n: NodesOwned<'n, C>) -> Self {
        StaticNodesOwned::new(n.0)
    }
}

impl<'n, C: Component> From<StaticNodesOwned<'n, C>> for NodesOwned<'n, C> {
    fn from(n: StaticNodesOwned<'n, C>) -> Self {
        NodesOwned::new(n.0)
    }
}

impl<'h, 'n: 'h, C: Component> From<Nodes<'h, 'n, C>> for StaticNodes<'h, 'n, C> {
    fn from(n: Nodes<'h, 'n, C>) -> Self {
        StaticNodes::new(n.0)
    }
}

impl<'h, 'n: 'h, C: Component> From<StaticNodes<'h, 'n, C>> for Nodes<'h, 'n, C> {
    fn from(n: StaticNodes<'h, 'n, C>) -> Self {
        Nodes::new(n.0)
    }
}

impl<'n, C: Component> From<HtmlElementUpdater<'n, C>> for HtmlNodesUpdater<'n, C> {
    fn from(r: HtmlElementUpdater<'n, C>) -> Self {
        let (r, m) = r.into_parts();
        Self {
            nodes_updater: From::from(r),
            _select_element_value_manager: m,
        }
    }
}

impl<'n, C: Component> From<HtmlElementUpdater<'n, C>> for NodesOwned<'n, C> {
    fn from(r: HtmlElementUpdater<'n, C>) -> Self {
        Self::new(From::from(r))
    }
}

impl<'n, C: Component> From<HtmlElementUpdater<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(r: HtmlElementUpdater<'n, C>) -> Self {
        Self::new(From::from(r))
    }
}

impl<'n, C: Component> From<AttributesOnly<'n, C>> for NodesOwned<'n, C> {
    fn from(r: AttributesOnly<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

impl<'n, C: Component> From<AttributesOnly<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(r: AttributesOnly<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

impl<'n, C: Component> From<StaticAttributesOnly<'n, C>> for NodesOwned<'n, C> {
    fn from(r: StaticAttributesOnly<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

impl<'n, C: Component> From<StaticAttributesOnly<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(r: StaticAttributesOnly<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

impl<'n, C: Component> From<StaticAttributes<'n, C>> for NodesOwned<'n, C> {
    fn from(r: StaticAttributes<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

impl<'n, C: Component> From<StaticAttributes<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(r: StaticAttributes<'n, C>) -> Self {
        Self::new(From::from(r.into_inner()))
    }
}

pub trait MethodsForHtmlElementContent<'n, C: Component>:
    ElementUpdaterMut<'n, C> + Into<NodesOwned<'n, C>> + Into<StaticNodesOwned<'n, C>>
{
    fn update_nodes(self) -> NodesOwned<'n, C> {
        self.into()
    }

    fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        self.into()
    }

    fn rfn(self, func: impl FnOnce(Nodes<C>)) -> NodesOwned<'n, C> {
        let mut n: NodesOwned<C> = self.into();
        let nodes = Nodes::new(&mut n.0);
        func(nodes);
        n
    }
}

impl<'n, C: Component> MethodsForHtmlElementContent<'n, C> for HtmlElementUpdater<'n, C> {}
impl<'n, C: Component> MethodsForHtmlElementContent<'n, C> for AttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForHtmlElementContent<'n, C> for StaticAttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForHtmlElementContent<'n, C> for StaticAttributes<'n, C> {}

impl<'h, 'n: 'h, C: Component> Nodes<'h, 'n, C> {
    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.nodes_updater.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.nodes_updater.comp()
    }

    pub fn static_nodes(self) -> StaticNodes<'h, 'n, C> {
        StaticNodes::new(self.0)
    }

    pub fn rfn(self, func: impl FnOnce(Nodes<C>)) -> Self {
        func(Nodes::new(self.0));
        self
    }

    /// Just a convenience method to create a single element. The original purpose
    /// is to use in list's render to reduce the indentation when only a single
    /// element is rendered per list entry.
    /// Without using `single_element`:
    /// ```rust
    /// fn render_a_list_entry(entry: &Entry, nodes: spair::Nodes<AppState>) {
    ///     nodes.div(|d| {
    ///         d.class("class-name")
    ///             .class_if(entry.is_something_on, "something")
    ///             .span(|s| {})
    ///             .span(|s| {});
    ///     });
    /// }
    /// ```
    /// When using `single_element`, the indentation reduces one level:
    /// ```rust
    /// fn render_a_list_entry(entry: &Entry, mut nodes: spair::Nodes<AppState>) {
    ///     nodes
    ///         .single_element("div")
    ///         .class("class-name")
    ///         .class_if(entry.is_something_on, "something")
    ///         .span(|s| {})
    ///         .span(|s| {});
    /// }
    /// ```
    pub fn single_element(&mut self, tag: &'static str) -> HtmlElementUpdater<C> {
        self.nodes_updater_mut()
            .get_element_updater(HtmlTag(tag))
            .expect_throw("Must be Some() because we are on Nodes")
    }
}

impl<'h, 'n: 'h, C: Component> StaticNodes<'h, 'n, C> {
    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.nodes_updater.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.nodes_updater.comp()
    }

    pub fn update_nodes(self) -> Nodes<'h, 'n, C> {
        Nodes::new(self.0)
    }
}

impl<'n, C: Component> NodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        StaticNodesOwned::new(self.0)
    }

    pub fn rfn(mut self, func: impl FnOnce(Nodes<C>)) -> Self {
        func(Nodes::new(&mut self.0));
        self
    }
}

impl<'n, C: Component> StaticNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn update_nodes(self) -> NodesOwned<'n, C> {
        NodesOwned::new(self.0)
    }
}

impl<'h, 'n: 'h, C: Component> UpdateHtmlElement<'n, C, Nodes<'h, 'n, C>> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> UpdateHtmlElement<'n, C, StaticNodes<'h, 'n, C>>
    for StaticNodes<'h, 'n, C>
{
}
impl<'n, C: Component> UpdateHtmlElement<'n, C, NodesOwned<'n, C>> for NodesOwned<'n, C> {}
impl<'n, C: Component> UpdateHtmlElement<'n, C, StaticNodesOwned<'n, C>>
    for StaticNodesOwned<'n, C>
{
}

impl<'h, 'n: 'h, C: Component> HemsHandMade<'n, C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsHandMade<'n, C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsHandMade<'n, C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsHandMade<'n, C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'h, 'n: 'h, C: Component> HemsForDistinctNames<'n, C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForDistinctNames<'n, C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForDistinctNames<'n, C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForDistinctNames<'n, C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'updater, C: Component> UpdateHtmlElement<'updater, C, NodesOwned<'updater, C>>
    for HtmlElementUpdater<'updater, C>
{
}
impl<'updater, C: Component> HemsHandMade<'updater, C> for HtmlElementUpdater<'updater, C> {
    type Output = NodesOwned<'updater, C>;
}
impl<'updater, C: Component> HemsForDistinctNames<'updater, C> for HtmlElementUpdater<'updater, C> {
    type Output = NodesOwned<'updater, C>;
}

impl<'updater, C: Component> UpdateHtmlElement<'updater, C, NodesOwned<'updater, C>>
    for StaticAttributes<'updater, C>
{
}
impl<'updater, C: Component> HemsHandMade<'updater, C> for StaticAttributes<'updater, C> {
    type Output = NodesOwned<'updater, C>;
}

impl<'updater, C: Component> HemsForDistinctNames<'updater, C> for StaticAttributes<'updater, C> {
    type Output = NodesOwned<'updater, C>;
}

pub struct HtmlMatchIfUpdater<'a, C: Component>(MatchIfUpdater<'a, C>);

impl<'a, C: Component> HtmlMatchIfUpdater<'a, C> {
    #[doc(hidden)]
    pub fn render_on_arm_index(self, index: TypeId) -> NodesOwned<'a, C> {
        NodesOwned(HtmlNodesUpdater {
            nodes_updater: self.0.render_on_arm_index(index),
            _select_element_value_manager: None, // How about a match_if inside a <select> element?
        })
    }

    pub fn state(&self) -> &'a C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }
}

#[cfg(test)]
mod tests {
    fn get_selected_value(nodes: &[crate::dom::Node]) -> Option<String> {
        let crate::dom::Node::Element(select_element) = nodes.first().unwrap() else {
            panic!("Expect an Element for <select>");
        };
        assert_eq!(
            crate::dom::ElementType::Select,
            select_element.element_type()
        );
        let select_element = select_element
            .ws_element()
            .unchecked_ref::<web_sys::HtmlSelectElement>();
        if select_element.selected_index() < 0 {
            None
        } else {
            Some(select_element.value())
        }
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn selected_value_on_select_element_start_with_none() {
        make_a_test_component! {
            type: Option<&'static str>;
            init: None;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.select(|s| {
                    s.selected_value(self.0.as_deref())
                        .option(|o| o.value("first-value").update_text("First Value").done())
                        .option(|o| o.value("second-value").update_text("Second Value").done())
                        .option(|o| o.value("third-value").update_text("Third Value").done());
                });
            }
        }

        let test = Test::set_up();
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
        test.update(Some("first-value"));
        assert_eq!(
            Some("first-value".to_string()),
            test.execute_on_nodes(get_selected_value)
        );
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn selected_value_on_select_element_start_with_none_string() {
        make_a_test_component! {
            type: Option<&'static str>;
            init: None;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.select(|s| {
                    s.selected_value(self.0.as_deref())
                        .option(|o| o.value("first-value".to_string()).update_text("First Value").done())
                        .option(|o| o.value("second-value".to_string()).update_text("Second Value").done())
                        .option(|o| o.value("third-value".to_string()).update_text("Third Value").done());
                });
            }
        }

        let test = Test::set_up();
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
        test.update(Some("first-value"));
        assert_eq!(
            Some("first-value".to_string()),
            test.execute_on_nodes(get_selected_value)
        );
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn selected_value_on_select_element_start_with_some() {
        make_a_test_component! {
            type: Option<&'static str>;
            init: Some("first-value");
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.select(|s| {
                    s.selected_value(self.0.as_deref())
                        .option(|o| o.value("first-value").update_text("First Value").done())
                        .option(|o| o.value("second-value").update_text("Second Value").done())
                        .option(|o| o.value("third-value").update_text("Third Value").done());
                });
            }
        }

        let test = Test::set_up();
        assert_eq!(
            Some("first-value".to_string()),
            test.execute_on_nodes(get_selected_value)
        );
        test.update(None);
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn selected_value_on_select_element_filled_by_list() {
        make_a_test_component! {
            type: Option<&'static str>;
            init: None;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.select(|s| {
                    s.selected_value(self.0.as_deref())
                        .list_clone(["first-value", "second-value", "third-value"].iter(), |value, mut nodes| {
                            nodes.single_element("option")
                                .value(*value)
                                .update_text(*value);
                    });
                });
            }
        }

        let test = Test::set_up();
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
        test.update(Some("first-value"));
        assert_eq!(
            Some("first-value".to_string()),
            test.execute_on_nodes(get_selected_value)
        );
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn selected_value_on_select_element_start_with_no_items() {
        struct Data {
            selected: Option<&'static str>,
            items: Vec<&'static str>,
        }
        make_a_test_component! {
            type: Data;
            init: Data {
                selected: None,
                items: Vec::new(),
            };
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.select(|s| {
                    s.selected_value(self.0.selected)
                        .list_clone(self.0.items.iter(), |value, mut nodes| {
                            nodes.single_element("option")
                                .value(*value)
                                .update_text(*value)
                                .static_text("--")
                                .update_text(*value);
                    });
                });
            }
        }

        let test = Test::set_up();
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
        test.update_with(|data| {
            data.items.push("first-value");
            data.items.push("second-value");
        });
        assert_eq!(None, test.execute_on_nodes(get_selected_value));
        test.update_with(|data| data.selected = Some("second-value"));
        assert_eq!(
            Some("second-value".to_string()),
            test.execute_on_nodes(get_selected_value)
        );
    }
}
