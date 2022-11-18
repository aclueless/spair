#[cfg(feature = "queue-render")]
use wasm_bindgen::UnwrapThrowExt;

use super::{
    SvgAttributesOnly, SvgElementRender, SvgElementUpdater, SvgRender, SvgStaticAttributes,
    SvgStaticAttributesOnly, SvgStaticRender, SvgTag,
};
use crate::{
    component::{Child, ChildComp, Comp, Component},
    render::base::{ElementUpdaterMut, MatchIfUpdater, NodesUpdater, NodesUpdaterMut},
};

#[cfg(feature = "queue-render")]
use crate::queue_render::val::QrVal;

pub trait UpdateSvgElement<C, O>: Sized
where
    C: Component,
    O: From<Self> + NodesUpdaterMut<C>,
{
    fn render_element(
        self,
        tag: &'static str,
        element_updater: impl FnOnce(SvgElementUpdater<C>),
    ) -> O {
        let mut this: O = self.into();
        let render = this.nodes_updater_mut();
        if render.require_update() {
            let e = render.get_element_updater(SvgTag(tag)).into();
            element_updater(e);
        }
        render.next_index();
        this
    }
}

pub trait SemsHandMade<C: Component>: Sized {
    type Output: From<Self> + NodesUpdaterMut<C>;
    fn match_if(self, f: impl FnOnce(SvgMatchIfUpdater<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        let mi = render.get_match_if_updater();
        let mi = SvgMatchIfUpdater(mi);
        f(mi);
        this
    }

    #[cfg(feature = "queue-render")]
    fn qr_match_if<T: 'static>(
        self,
        value: &QrVal<T>,
        f: impl Fn(&T, SvgMatchIfUpdater<C>) + 'static,
    ) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.nodes_updater_mut();
        if let Some(mi) = render.create_qr_match_if(move |t, mi| {
            let mi = SvgMatchIfUpdater(mi);
            f(t, mi);
        }) {
            value
                .content()
                .try_borrow_mut()
                .expect_throw("render::svg::nodes::SemsHandMade::qr_match_if")
                .add_render(Box::new(mi));
        }
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

#[cfg(test)]
use crate::render::svg::TestSvgMethods;

make_trait_for_element_methods! {
    TestStructs: (TestSvgMethods)
    TraitName: SemsForDistinctNames
    UpdateElementTraitName: UpdateSvgElement
    ElementUpdaterType: SvgElementUpdater
    elements:
        // https://developer.mozilla.org/en-US/docs/Web/SVG/Element
        a
        animate
        animate_motion "animateMotion"
        animate_transform "animateTransform"
        circle
        // ambiguous
        // clip_path "clipPath"
        defs
        desc
        discard
        ellipse
        fe_blend "feBlend"
        fe_color_matrix "feColorMatrix"
        fe_component_transfer "feComponentTransfer"
        fe_composite "feComposite"
        fe_convolve_matrix "feConvolveMatrix"
        fe_diffuse_lighting "feDiffuseLighting"
        fe_displacement_map "feDisplacementMap"
        fe_distant_light "feDistantLight"
        fe_drop_shadow "feDropShadow"
        fe_flood "feFlood"
        fe_func_a "feFuncA"
        fe_func_b "feFuncB"
        fe_func_g "feFuncG"
        fe_func_r "feFuncR"
        fe_gaussian_blur "feGaussianBlur"
        fe_image "feImage"
        fe_merge "feMerge"
        fe_merge_node "feMergeNode"
        fe_morphology "feMorphology"
        fe_offset "feOffset"
        fe_point_light "fePointLight"
        fe_specular_lighting "feSpecularLighting"
        fe_spot_light "feSpotLight"
        fe_tile "feTile"
        fe_turbulence "feTurbulence"
        filter
        foreign_object "foreignObject"
        g
        hatch
        hatchpath
        image
        line
        linear_gradient "linearGradient"
        marker
        // ambiguous
        // mask
        mesh
        meshgradient
        meshpatch
        meshrow
        metadata
        mpath
        // ambiguous
        // path
        pattern
        polygon
        polyline
        radial_gradient "radialGradient"
        rect
        //script ??
        set
        solidcolor
        stop
        style_element "style" // conflict with attribute with the same name
        svg
        switch
        symbol
        text
        text_path "textPath"
        title
        tspan
        r#use "use"
        view
}

pub struct SvgNodesOwned<'n, C: Component>(NodesUpdater<'n, C>);
pub struct SvgStaticNodesOwned<'n, C: Component>(NodesUpdater<'n, C>);
pub struct SvgNodes<'h, 'n: 'h, C: Component>(&'h mut NodesUpdater<'n, C>);
pub struct SvgStaticNodes<'h, 'n: 'h, C: Component>(&'h mut NodesUpdater<'n, C>);

impl<'n, C: Component> SvgNodesOwned<'n, C> {
    fn new(mut r: NodesUpdater<'n, C>) -> Self {
        r.set_update_mode();
        Self(r)
    }
}

impl<'n, C: Component> SvgStaticNodesOwned<'n, C> {
    fn new(mut r: NodesUpdater<'n, C>) -> Self {
        r.set_static_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> SvgNodes<'h, 'n, C> {
    fn new(r: &'h mut NodesUpdater<'n, C>) -> Self {
        r.set_update_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> SvgStaticNodes<'h, 'n, C> {
    fn new(r: &'h mut NodesUpdater<'n, C>) -> Self {
        r.set_static_mode();
        Self(r)
    }
}

impl<'n, C: Component> From<SvgNodesOwned<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(n: SvgNodesOwned<'n, C>) -> Self {
        SvgStaticNodesOwned::new(n.0)
    }
}

impl<'n, C: Component> From<SvgStaticNodesOwned<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(n: SvgStaticNodesOwned<'n, C>) -> Self {
        SvgNodesOwned::new(n.0)
    }
}

impl<'h, 'n: 'h, C: Component> From<SvgNodes<'h, 'n, C>> for SvgStaticNodes<'h, 'n, C> {
    fn from(n: SvgNodes<'h, 'n, C>) -> Self {
        SvgStaticNodes::new(n.0)
    }
}

impl<'h, 'n: 'h, C: Component> From<SvgStaticNodes<'h, 'n, C>> for SvgNodes<'h, 'n, C> {
    fn from(n: SvgStaticNodes<'h, 'n, C>) -> Self {
        SvgNodes::new(n.0)
    }
}

impl<'n, C: Component> NodesUpdaterMut<C> for SvgNodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.0
    }
}

impl<'n, C: Component> NodesUpdaterMut<C> for SvgStaticNodesOwned<'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        &mut self.0
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<C> for SvgNodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        self.0
    }
}

impl<'h, 'n: 'h, C: Component> NodesUpdaterMut<C> for SvgStaticNodes<'h, 'n, C> {
    fn nodes_updater_mut(&mut self) -> &'n mut NodesUpdater<C> {
        self.0
    }
}

impl<'n, C: Component> From<SvgElementUpdater<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(r: SvgElementUpdater<'n, C>) -> Self {
        let r = r.into_inner();
        Self(From::from(r))
    }
}
impl<'n, C: Component> From<SvgStaticAttributes<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(r: SvgStaticAttributes<'n, C>) -> Self {
        let r = r.into_inner();
        Self(From::from(r))
    }
}

impl<'n, C: Component> UpdateSvgElement<C, SvgNodesOwned<'n, C>> for SvgElementUpdater<'n, C> {}
impl<'n, C: Component> UpdateSvgElement<C, SvgNodesOwned<'n, C>> for SvgStaticAttributes<'n, C> {}

impl<'h, 'n: 'h, C: Component> UpdateSvgElement<C, SvgNodes<'h, 'n, C>> for SvgNodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> UpdateSvgElement<C, SvgStaticNodes<'h, 'n, C>>
    for SvgStaticNodes<'h, 'n, C>
{
}
impl<'n, C: Component> UpdateSvgElement<C, SvgNodesOwned<'n, C>> for SvgNodesOwned<'n, C> {}
impl<'n, C: Component> UpdateSvgElement<C, SvgStaticNodesOwned<'n, C>>
    for SvgStaticNodesOwned<'n, C>
{
}

impl<'h, 'n: 'h, C: Component> SemsHandMade<C> for SvgNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> SemsHandMade<C> for SvgStaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsHandMade<C> for SvgNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsHandMade<C> for SvgStaticNodesOwned<'n, C> {
    type Output = Self;
}

impl<'h, 'n: 'h, C: Component> SemsForDistinctNames<C> for SvgNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'h, 'n: 'h, C: Component> SemsForDistinctNames<C> for SvgStaticNodes<'h, 'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgStaticNodesOwned<'n, C> {
    type Output = Self;
}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgStaticAttributes<'n, C> {
    type Output = SvgNodesOwned<'n, C>;
}
impl<'n, C: Component> SemsForDistinctNames<C> for SvgElementUpdater<'n, C> {
    type Output = SvgNodesOwned<'n, C>;
}

impl<'h, 'n: 'h, C: Component> SvgNodes<'h, 'n, C> {
    pub(super) fn update_text(self, text: &str) {
        self.0.update_text(text);
    }

    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }

    pub fn static_nodes(self) -> SvgStaticNodes<'h, 'n, C> {
        SvgStaticNodes::new(self.0)
    }

    pub fn rupdate(self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn rstatic(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(self.0);
        render.render(n);
        self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: SvgElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }

    pub fn rfn(self, func: impl FnOnce(SvgNodes<C>)) -> Self {
        let n = SvgNodes::new(self.0);
        func(n);
        self
    }
}

impl<'h, 'n: 'h, C: Component> SvgStaticNodes<'h, 'n, C> {
    pub(super) fn static_text(self, text: &str) {
        self.0.static_text(text);
    }

    pub fn done(self) {}

    pub fn state(&self) -> &'n C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }

    pub fn update_nodes(self) -> SvgNodes<'h, 'n, C> {
        SvgNodes::new(self.0)
    }

    // No .rupdate() on a `SvgStaticNodes`
    // pub fn rupdate(mut self, render: impl SvgRender<C>) -> Self {}

    pub fn rstatic(self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(self.0);
        render.render(n);
        //self.nodes_updater_mut().set_static_mode();
        self
    }

    pub fn relement<R: SvgElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }
}

impl<'n, C: Component> SvgNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn static_nodes(self) -> SvgStaticNodesOwned<'n, C> {
        SvgStaticNodesOwned::new(self.0)
    }

    pub fn rupdate(mut self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(&mut self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn rstatic(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(&mut self.0);
        render.render(n);
        self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: SvgElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }

    pub fn rfn(mut self, func: impl FnOnce(SvgNodes<C>)) -> Self {
        let n = SvgNodes::new(&mut self.0);
        func(n);
        self
    }
}

impl<'n, C: Component> SvgStaticNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn update_nodes(self) -> SvgNodesOwned<'n, C> {
        SvgNodesOwned::new(self.0)
    }

    pub fn rupdate(mut self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(&mut self.0);
        render.render(n);
        self.nodes_updater_mut().set_static_mode();
        self
    }

    pub fn rstatic(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(&mut self.0);
        render.render(n);
        //self.nodes_updater_mut().set_update_mode();
        self
    }

    pub fn relement<R: SvgElementRender<C>>(self, render: R) -> Self {
        self.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }
}

pub trait MethodsForSvgElementContent<'n, C: Component>:
    ElementUpdaterMut<C> + Into<SvgNodesOwned<'n, C>> + Into<SvgStaticNodesOwned<'n, C>>
{
    fn update_nodes(self) -> SvgNodesOwned<'n, C> {
        self.into()
    }

    fn static_nodes(self) -> SvgStaticNodesOwned<'n, C> {
        self.into()
    }

    fn rupdate(self, render: impl SvgRender<C>) -> SvgNodesOwned<'n, C> {
        let n: SvgNodesOwned<C> = self.into();
        n.rupdate(render)
    }

    fn rstatic(self, render: impl SvgStaticRender<C>) -> SvgNodesOwned<'n, C> {
        let n: SvgNodesOwned<C> = self.into();
        n.rstatic(render)
    }

    fn relement<R: SvgElementRender<C>>(self, render: R) -> SvgNodesOwned<'n, C> {
        let n: SvgNodesOwned<C> = self.into();
        n.render_element(R::ELEMENT_TAG, |e| render.render(e))
    }

    fn rfn(self, func: impl FnOnce(SvgNodes<C>)) -> SvgNodesOwned<'n, C> {
        let mut n: SvgNodesOwned<C> = self.into();
        let nodes = SvgNodes::new(&mut n.0);
        func(nodes);
        n
    }
}

impl<'n, C: Component> From<SvgElementUpdater<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(r: SvgElementUpdater<'n, C>) -> Self {
        SvgStaticNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> From<SvgAttributesOnly<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(r: SvgAttributesOnly<'n, C>) -> Self {
        SvgStaticNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> From<SvgAttributesOnly<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(r: SvgAttributesOnly<'n, C>) -> Self {
        SvgNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> From<SvgStaticAttributesOnly<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(r: SvgStaticAttributesOnly<'n, C>) -> Self {
        SvgStaticNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> From<SvgStaticAttributesOnly<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(r: SvgStaticAttributesOnly<'n, C>) -> Self {
        SvgNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> From<SvgStaticAttributes<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(r: SvgStaticAttributes<'n, C>) -> Self {
        SvgStaticNodesOwned::new(From::from(r.into_inner()))
    }
}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgElementUpdater<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgAttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgStaticAttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgStaticAttributes<'n, C> {}

pub struct SvgMatchIfUpdater<'a, C: Component>(MatchIfUpdater<'a, C>);

impl<'a, C: Component> SvgMatchIfUpdater<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> SvgNodesOwned<'a, C> {
        SvgNodesOwned::new(self.0.render_on_arm_index(index))
    }

    pub fn state(&self) -> &'a C {
        self.0.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.0.comp()
    }
}
