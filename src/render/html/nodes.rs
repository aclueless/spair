use super::{
    AmbiguousHtmlElementMethods, AttributesOnly, HtmlElementRender, HtmlNameSpace, Render,
    SelectElementValueManager, StaticAttributes, StaticAttributesOnly, StaticRender,
};
use crate::component::{ChildComp, Comp, Component};
use crate::render::base::{ElementRenderMut, MatchIfRender, NodeListRender, NodeListRenderMut};

pub struct HtmlNodeListRender<'n, C: Component> {
    node_list_render: NodeListRender<'n, C>,
    // Just keep this value until the completion of the build of the whole node list
    // After done building the node list, this value will be dropped. The Drop::drop method
    // will execute setting value for the <select> element
    _select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'n, C: Component> NodeListRenderMut<C> for HtmlNodeListRender<'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.node_list_render
    }
}

pub trait SpecializedHtmlElementMethods<C: Component>: Sized {
    type Output: From<Self> + NodeListRenderMut<C>;

    fn line_break(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.node_list_render_mut();
        if render.require_render() {
            render.get_element_render::<HtmlNameSpace>("br");
        }
        render.next_index();
        this
    }

    fn horizontal_line(self) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.node_list_render_mut();
        if render.require_render() {
            render.get_element_render::<HtmlNameSpace>("hr");
        }
        render.next_index();
        this
    }

    fn match_if(self, f: impl FnOnce(HtmlMatchIfRender<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.node_list_render_mut();
        let mi = render.get_match_if_updater();
        let mi = HtmlMatchIfRender(mi);
        f(mi);
        this
    }
}

pub trait RenderHtmlElement<C, O>: Sized
where
    C: Component,
    O: From<Self> + NodeListRenderMut<C>,
{
    fn render_element(self, tag: &str, element_render: impl FnOnce(HtmlElementRender<C>)) -> O {
        let mut this: O = self.into();
        let render = this.node_list_render_mut();
        if render.require_render() {
            let e = render.get_element_render::<HtmlNameSpace>(tag).into();
            element_render(e);
        }
        render.next_index();
        this
    }
}

#[cfg(test)]
use crate::render::html::{AllAttributes, AllElements};

make_trait_for_element_methods! {
    TestStructs: (AllElements AllAttributes)
    TraitName: HtmlElementMethods
    RenderElementTraitName: RenderHtmlElement
    ElementRenderType: HtmlElementRender
    elements:
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

pub struct NodesOwned<'n, C: Component>(HtmlNodeListRender<'n, C>);
pub struct StaticNodesOwned<'n, C: Component>(HtmlNodeListRender<'n, C>);
pub struct Nodes<'h, 'n: 'h, C: Component>(&'h mut HtmlNodeListRender<'n, C>);
pub struct StaticNodes<'h, 'n: 'h, C: Component>(&'h mut HtmlNodeListRender<'n, C>);

impl<'n, C: Component> NodesOwned<'n, C> {
    fn new(mut r: HtmlNodeListRender<'n, C>) -> Self {
        r.node_list_render.set_update_mode();
        Self(r)
    }
}

impl<'n, C: Component> StaticNodesOwned<'n, C> {
    fn new(mut r: HtmlNodeListRender<'n, C>) -> Self {
        r.node_list_render.set_static_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> Nodes<'h, 'n, C> {
    fn new(r: &'h mut HtmlNodeListRender<'n, C>) -> Self {
        r.node_list_render.set_update_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> StaticNodes<'h, 'n, C> {
    fn new(r: &'h mut HtmlNodeListRender<'n, C>) -> Self {
        r.node_list_render.set_static_mode();
        Self(r)
    }
}

impl<'n, C: Component> NodeListRenderMut<C> for NodesOwned<'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0.node_list_render
    }
}

impl<'n, C: Component> NodeListRenderMut<C> for StaticNodesOwned<'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0.node_list_render
    }
}

impl<'h, 'n: 'h, C: Component> NodeListRenderMut<C> for Nodes<'h, 'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0.node_list_render
    }
}

impl<'h, 'n: 'h, C: Component> NodeListRenderMut<C> for StaticNodes<'h, 'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0.node_list_render
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

impl<'n, C: Component> From<HtmlElementRender<'n, C>> for HtmlNodeListRender<'n, C> {
    fn from(r: HtmlElementRender<'n, C>) -> Self {
        let (r, m) = r.into_parts();
        Self {
            node_list_render: From::from(r),
            _select_element_value_manager: m,
        }
    }
}

impl<'n, C: Component> From<HtmlElementRender<'n, C>> for NodesOwned<'n, C> {
    fn from(r: HtmlElementRender<'n, C>) -> Self {
        Self::new(From::from(r))
    }
}

impl<'n, C: Component> From<HtmlElementRender<'n, C>> for StaticNodesOwned<'n, C> {
    fn from(r: HtmlElementRender<'n, C>) -> Self {
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

pub trait NodeAndTextMethodsOnAttributeX<'n, C: Component>:
    ElementRenderMut<C> + Into<NodesOwned<'n, C>> + Into<StaticNodesOwned<'n, C>>
{
    fn update_nodes(self) -> NodesOwned<'n, C> {
        self.into()
    }

    fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        self.into()
    }

    fn update_render(self, render: impl Render<C>) -> NodesOwned<'n, C> {
        let n: NodesOwned<C> = self.into();
        n.update_render(render)
    }

    fn static_render(self, render: impl StaticRender<C>) -> NodesOwned<'n, C> {
        let n: NodesOwned<C> = self.into();
        n.static_render(render)
    }

    fn component<CC: Component>(mut self, child: &ChildComp<CC>) {
        self.element_render_mut().component(child)
    }
}

impl<'n, C: Component> NodeAndTextMethodsOnAttributeX<'n, C> for HtmlElementRender<'n, C> {}
impl<'n, C: Component> NodeAndTextMethodsOnAttributeX<'n, C> for AttributesOnly<'n, C> {}
impl<'n, C: Component> NodeAndTextMethodsOnAttributeX<'n, C> for StaticAttributesOnly<'n, C> {}
impl<'n, C: Component> NodeAndTextMethodsOnAttributeX<'n, C> for StaticAttributes<'n, C> {}

impl<'h, 'n: 'h, C: Component> Nodes<'h, 'n, C> {
    pub(super) fn update_text(self, text: &str) {
        self.0.node_list_render.update_text(text);
    }

    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.node_list_render.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.node_list_render.comp()
    }

    pub fn static_nodes(self) -> StaticNodes<'h, 'n, C> {
        StaticNodes::new(self.0)
    }

    pub fn update_render(self, render: impl Render<C>) -> Self {
        let n = Nodes::new(self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }

    pub fn static_render(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(self.0);
        render.render(n);
        self.node_list_render_mut().set_update_mode();
        self
    }
}

impl<'h, 'n: 'h, C: Component> StaticNodes<'h, 'n, C> {
    pub(super) fn static_text(self, text: &str) {
        self.0.node_list_render.static_text(text);
    }

    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.node_list_render.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.node_list_render.comp()
    }

    pub fn update_nodes(self) -> Nodes<'h, 'n, C> {
        Nodes::new(self.0)
    }

    // No .update_render() on a `StaticNodes`
    // pub fn update_render(mut self, render: impl Render<C>) -> Self {}

    pub fn static_render(self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(self.0);
        render.render(n);
        //self.node_list_render_mut().set_static_mode();
        self
    }
}

impl<'n, C: Component> NodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn static_nodes(self) -> StaticNodesOwned<'n, C> {
        StaticNodesOwned::new(self.0)
    }

    pub fn update_render(mut self, render: impl Render<C>) -> Self {
        let n = Nodes::new(&mut self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }

    pub fn static_render(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(&mut self.0);
        render.render(n);
        self.node_list_render_mut().set_update_mode();
        self
    }
}

impl<'n, C: Component> StaticNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn update_nodes(self) -> NodesOwned<'n, C> {
        NodesOwned::new(self.0)
    }

    pub fn update_render(mut self, render: impl Render<C>) -> Self {
        let n = Nodes::new(&mut self.0);
        render.render(n);
        self.node_list_render_mut().set_static_mode();
        self
    }

    pub fn static_render(mut self, render: impl StaticRender<C>) -> Self {
        let n = StaticNodes::new(&mut self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }
}

impl<'h, 'n: 'h, C: Component> RenderHtmlElement<C, Nodes<'h, 'n, C>> for Nodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> RenderHtmlElement<C, StaticNodes<'h, 'n, C>>
    for StaticNodes<'h, 'n, C>
{
}
impl<'n, C: Component> RenderHtmlElement<C, NodesOwned<'n, C>> for NodesOwned<'n, C> {}
impl<'n, C: Component> RenderHtmlElement<C, StaticNodesOwned<'n, C>> for StaticNodesOwned<'n, C> {}

impl<'h, 'n: 'h, C: Component> AmbiguousHtmlElementMethods<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> AmbiguousHtmlElementMethods<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> AmbiguousHtmlElementMethods<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> AmbiguousHtmlElementMethods<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'h, 'n: 'h, C: Component> SpecializedHtmlElementMethods<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> SpecializedHtmlElementMethods<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SpecializedHtmlElementMethods<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SpecializedHtmlElementMethods<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'h, 'n: 'h, C: Component> HtmlElementMethods<C> for Nodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> HtmlElementMethods<C> for StaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HtmlElementMethods<C> for NodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> HtmlElementMethods<C> for StaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'er, C: Component> RenderHtmlElement<C, NodesOwned<'er, C>> for HtmlElementRender<'er, C> {}
impl<'er, C: Component> SpecializedHtmlElementMethods<C> for HtmlElementRender<'er, C> {
    type Output = NodesOwned<'er, C>;
}
impl<'er, C: Component> HtmlElementMethods<C> for HtmlElementRender<'er, C> {
    type Output = NodesOwned<'er, C>;
}

impl<'er, C: Component> RenderHtmlElement<C, NodesOwned<'er, C>> for StaticAttributes<'er, C> {}
impl<'er, C: Component> SpecializedHtmlElementMethods<C> for StaticAttributes<'er, C> {
    type Output = NodesOwned<'er, C>;
}
impl<'er, C: Component> HtmlElementMethods<C> for StaticAttributes<'er, C> {
    type Output = NodesOwned<'er, C>;
}

pub struct HtmlMatchIfRender<'a, C: Component>(MatchIfRender<'a, C>);

impl<'a, C: Component> HtmlMatchIfRender<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> NodesOwned<'a, C> {
        NodesOwned(HtmlNodeListRender {
            node_list_render: self.0.render_on_arm_index(index),
            _select_element_value_manager: None, // How about a match_if inside a <select> element?
        })
    }
}
