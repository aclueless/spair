#[cfg(feature = "queue-render")]
use wasm_bindgen::UnwrapThrowExt;

use super::{
    AttributesOnly, ElementRender, HtmlElementUpdater, HtmlTag, Render, SelectElementValueManager,
    StaticAttributes, StaticAttributesOnly, StaticRender,
};
#[cfg(feature = "svg")]
use crate::render::svg::{SvgElementUpdater, SvgTag};
use crate::{
    component::{Child, ChildComp, Comp, Component},
    render::base::{ElementUpdaterMut, MatchIfUpdater, NodesUpdater, NodesUpdaterMut},
};

#[cfg(feature = "queue-render")]
use crate::queue_render::value::QrVal;

pub struct HtmlNodesUpdater<'n, C: Component> {
    nodes_updater: NodesUpdater<'n, C>,
    // Just keep this value until the completion of the build of the whole node list
    // After done building the node list, this value will be dropped. The Drop::drop method
    // will execute setting value for the <select> element
    _select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'n, C: Component> NodesUpdaterMut<C> for HtmlNodesUpdater<'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.nodes_updater
    }
}

pub trait HemsHandMade<C: Component>: Sized {
    type Output: From<Self> + NodesUpdaterMut<C>;

    fn line_break(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            render.get_element_updater(HtmlTag("br"));
        }
        render.next_index();
        this
    }

    fn horizontal_line(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            render.get_element_updater(HtmlTag("hr"));
        }
        render.next_index();
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
    fn svg(self, f: impl FnOnce(SvgElementUpdater<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            let r = render.get_element_updater(SvgTag("svg"));
            f(r.into())
        }
        render.next_index();
        this
    }

    fn component_ref<CC: Component>(self, child: &ChildComp<CC>) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            render.component_ref(child);
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
}

pub trait UpdateHtmlElement<C, O>: Sized
where
    C: Component,
    O: From<Self> + NodesUpdaterMut<C>,
{
    fn render_element(
        self,
        tag: &'static str,
        element_updater: impl FnOnce(HtmlElementUpdater<C>),
    ) -> O {
        let mut this: O = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            let e = render.get_element_updater(HtmlTag(tag)).into();
            element_updater(e);
        }
        render.next_index();
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
    fn new(r: &'h mut HtmlNodesUpdater<'n, C>) -> Self {
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

impl<'n, C: Component> NodesUpdaterMut<C> for NodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.0.nodes_updater
    }
}

impl<'n, C: Component> NodesUpdaterMut<C> for StaticNodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.0.nodes_updater
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<C> for Nodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.0.nodes_updater
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<C> for StaticNodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
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
    ElementUpdaterMut<C> + Into<NodesOwned<'n, C>> + Into<StaticNodesOwned<'n, C>>
{
    fn update_nodes(self) -> NodesOwned<'n, C> {
        self.into()
    }

    fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        self.into()
    }

    fn rupdate(self, render: impl Render<C>) -> NodesOwned<'n, C> {
        let n: NodesOwned<C> = self.into();
        n.rupdate(render)
    }

    fn rstatic(self, render: impl StaticRender<C>) -> NodesOwned<'n, C> {
        let n: NodesOwned<C> = self.into();
        n.rstatic(render)
    }

    fn relement<R: ElementRender<C>>(self, render: R) -> NodesOwned<'n, C> {
        let n: NodesOwned<C> = self.into();
        n.render_element(R::ELEMENT_TAG, |e| render.render(e))
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
    pub(super) fn update_text(self, text: &str) {
        self.0.nodes_updater.update_text(text);
    }

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

    pub fn rupdate(self, render: impl Render<C>) -> Self {
        let n = Nodes::new(self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn rstatic(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(self.0);
        render.render(n);
        self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: ElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }

    pub fn rfn(self, func: impl FnOnce(Nodes<C>)) -> Self {
        let n = Nodes::new(self.0);
        func(n);
        self
    }
}

impl<'h, 'n: 'h, C: Component> StaticNodes<'h, 'n, C> {
    pub(super) fn static_text(self, text: &str) {
        self.0.nodes_updater.static_text(text);
    }

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

    // No .rupdate() on a `StaticNodes`???
    // pub fn rupdate(mut self, render: impl Render<C>) -> Self {}

    pub fn rstatic(self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(self.0);
        render.render(n);
        //self.nodes_updater_mut().set_static_mode();
        self
    }
}

impl<'n, C: Component> NodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        StaticNodesOwned::new(self.0)
    }

    pub fn rupdate(mut self, render: impl Render<C>) -> Self {
        let n = Nodes::new(&mut self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn rstatic(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(&mut self.0);
        render.render(n);
        self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: ElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }

    pub fn rfn(mut self, func: impl FnOnce(Nodes<C>)) -> Self {
        let n = Nodes::new(&mut self.0);
        func(n);
        self
    }
}

impl<'n, C: Component> StaticNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn update_nodes(self) -> NodesOwned<'n, C> {
        NodesOwned::new(self.0)
    }

    pub fn rupdate(mut self, render: impl Render<C>) -> Self {
        let n = Nodes::new(&mut self.0);
        render.render(n);
        self.nodes_updater_mut().set_static_mode();
        self
    }

    pub fn rstatic(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(&mut self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: ElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }
}

impl<'h, 'n: 'h, C: Component> UpdateHtmlElement<C, Nodes<'h, 'n, C>> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> UpdateHtmlElement<C, StaticNodes<'h, 'n, C>>
    for StaticNodes<'h, 'n, C>
{
}
impl<'n, C: Component> UpdateHtmlElement<C, NodesOwned<'n, C>> for NodesOwned<'n, C> {}
impl<'n, C: Component> UpdateHtmlElement<C, StaticNodesOwned<'n, C>> for StaticNodesOwned<'n, C> {}

impl<'h, 'n: 'h, C: Component> HemsHandMade<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsHandMade<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsHandMade<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsHandMade<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'h, 'n: 'h, C: Component> HemsForDistinctNames<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HemsForDistinctNames<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForDistinctNames<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HemsForDistinctNames<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'er, C: Component> UpdateHtmlElement<C, NodesOwned<'er, C>> for HtmlElementUpdater<'er, C> {}
impl<'er, C: Component> HemsHandMade<C> for HtmlElementUpdater<'er, C> {
    type Output = NodesOwned<'er, C>;
}
impl<'er, C: Component> HemsForDistinctNames<C> for HtmlElementUpdater<'er, C> {
    type Output = NodesOwned<'er, C>;
}

impl<'er, C: Component> UpdateHtmlElement<C, NodesOwned<'er, C>> for StaticAttributes<'er, C> {}
impl<'er, C: Component> HemsHandMade<C> for StaticAttributes<'er, C> {
    type Output = NodesOwned<'er, C>;
}
impl<'er, C: Component> HemsForDistinctNames<C> for StaticAttributes<'er, C> {
    type Output = NodesOwned<'er, C>;
}

pub struct HtmlMatchIfUpdater<'a, C: Component>(MatchIfUpdater<'a, C>);

impl<'a, C: Component> HtmlMatchIfUpdater<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> NodesOwned<'a, C> {
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
