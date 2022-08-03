use super::{
    SvgAttributesOnly, SvgElementRender, SvgNameSpace, SvgRender, SvgStaticAttributes,
    SvgStaticAttributesOnly, SvgStaticRender,
};
use crate::component::{ChildComp, Comp, Component};
use crate::render::base::{ElementRenderMut, MatchIfRender, NodeListRender, NodeListRenderMut};

pub trait RenderSvgElement<C, O>: Sized
where
    C: Component,
    O: From<Self> + NodeListRenderMut<C>,
{
    fn render_element(self, tag: &str, element_render: impl FnOnce(SvgElementRender<C>)) -> O {
        let mut this: O = self.into();
        let render = this.node_list_render_mut();
        if render.require_render() {
            let e = render.get_element_render::<SvgNameSpace>(tag).into();
            element_render(e);
        }
        render.next_index();
        this
    }
}

pub trait SemsHandMade<C: Component>: Sized {
    type Output: From<Self> + NodeListRenderMut<C>;
    fn match_if(self, f: impl FnOnce(SvgMatchIfRender<C>)) -> Self::Output {
        let mut this: Self::Output = self.into();
        let render = this.node_list_render_mut();
        let mi = render.get_match_if_updater();
        let mi = SvgMatchIfRender(mi);
        f(mi);
        this
    }
}

#[cfg(test)]
use crate::render::svg::TestSvgMethods;

make_trait_for_element_methods! {
    TestStructs: (TestSvgMethods)
    TraitName: SemsForDistinctNames
    RenderElementTraitName: RenderSvgElement
    ElementRenderType: SvgElementRender
    elements:
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

pub struct SvgNodesOwned<'n, C: Component>(NodeListRender<'n, C>);
pub struct SvgStaticNodesOwned<'n, C: Component>(NodeListRender<'n, C>);
pub struct SvgNodes<'h, 'n: 'h, C: Component>(&'h mut NodeListRender<'n, C>);
pub struct SvgStaticNodes<'h, 'n: 'h, C: Component>(&'h mut NodeListRender<'n, C>);

impl<'n, C: Component> SvgNodesOwned<'n, C> {
    fn new(mut r: NodeListRender<'n, C>) -> Self {
        r.set_update_mode();
        Self(r)
    }
}

impl<'n, C: Component> SvgStaticNodesOwned<'n, C> {
    fn new(mut r: NodeListRender<'n, C>) -> Self {
        r.set_static_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> SvgNodes<'h, 'n, C> {
    fn new(r: &'h mut NodeListRender<'n, C>) -> Self {
        r.set_update_mode();
        Self(r)
    }
}

impl<'h, 'n: 'h, C: Component> SvgStaticNodes<'h, 'n, C> {
    fn new(r: &'h mut NodeListRender<'n, C>) -> Self {
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

impl<'n, C: Component> NodeListRenderMut<C> for SvgNodesOwned<'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0
    }
}

impl<'n, C: Component> NodeListRenderMut<C> for SvgStaticNodesOwned<'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0
    }
}

impl<'h, 'n: 'h, C: Component> NodeListRenderMut<C> for SvgNodes<'h, 'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0
    }
}

impl<'h, 'n: 'h, C: Component> NodeListRenderMut<C> for SvgStaticNodes<'h, 'n, C> {
    fn node_list_render_mut(&mut self) -> &'n mut NodeListRender<C> {
        &mut self.0
    }
}

impl<'n, C: Component> From<SvgElementRender<'n, C>> for SvgNodesOwned<'n, C> {
    fn from(r: SvgElementRender<'n, C>) -> Self {
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

impl<'n, C: Component> RenderSvgElement<C, SvgNodesOwned<'n, C>> for SvgElementRender<'n, C> {}
impl<'n, C: Component> RenderSvgElement<C, SvgNodesOwned<'n, C>> for SvgStaticAttributes<'n, C> {}

impl<'h, 'n: 'h, C: Component> RenderSvgElement<C, SvgNodes<'h, 'n, C>> for SvgNodes<'h, 'n, C> {}
impl<'h, 'n: 'h, C: Component> RenderSvgElement<C, SvgStaticNodes<'h, 'n, C>>
    for SvgStaticNodes<'h, 'n, C>
{
}
impl<'n, C: Component> RenderSvgElement<C, SvgNodesOwned<'n, C>> for SvgNodesOwned<'n, C> {}
impl<'n, C: Component> RenderSvgElement<C, SvgStaticNodesOwned<'n, C>>
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

    pub fn update_render(self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }

    pub fn static_render(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(self.0);
        render.render(n);
        self.node_list_render_mut().set_update_mode();
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

    // No .update_render() on a `SvgStaticNodes`
    // pub fn update_render(mut self, render: impl SvgRender<C>) -> Self {}

    pub fn static_render(self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(self.0);
        render.render(n);
        //self.node_list_render_mut().set_static_mode();
        self
    }
}

impl<'n, C: Component> SvgNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn static_nodes(self) -> SvgStaticNodesOwned<'n, C> {
        SvgStaticNodesOwned::new(self.0)
    }

    pub fn update_render(mut self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(&mut self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }

    pub fn static_render(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(&mut self.0);
        render.render(n);
        self.node_list_render_mut().set_update_mode();
        self
    }
}

impl<'n, C: Component> SvgStaticNodesOwned<'n, C> {
    pub fn done(self) {}

    pub fn update_nodes(self) -> SvgNodesOwned<'n, C> {
        SvgNodesOwned::new(self.0)
    }

    pub fn update_render(mut self, render: impl SvgRender<C>) -> Self {
        let n = SvgNodes::new(&mut self.0);
        render.render(n);
        self.node_list_render_mut().set_static_mode();
        self
    }

    pub fn static_render(mut self, render: impl SvgStaticRender<C>) -> Self {
        let n = SvgStaticNodes::new(&mut self.0);
        render.render(n);
        //self.node_list_render_mut().set_update_mode();
        self
    }
}

pub trait MethodsForSvgElementContent<'n, C: Component>:
    ElementRenderMut<C> + Into<SvgNodesOwned<'n, C>> + Into<SvgStaticNodesOwned<'n, C>>
{
    fn update_nodes(self) -> SvgNodesOwned<'n, C> {
        self.into()
    }

    fn static_nodes(self) -> SvgStaticNodesOwned<'n, C> {
        self.into()
    }

    fn update_render(self, render: impl SvgRender<C>) -> SvgNodesOwned<'n, C> {
        let n: SvgNodesOwned<C> = self.into();
        n.update_render(render)
    }

    fn static_render(self, render: impl SvgStaticRender<C>) -> SvgNodesOwned<'n, C> {
        let n: SvgNodesOwned<C> = self.into();
        n.static_render(render)
    }

    fn component<CC: Component>(mut self, child: &ChildComp<CC>) {
        self.element_render_mut().component(child)
    }
}

impl<'n, C: Component> From<SvgElementRender<'n, C>> for SvgStaticNodesOwned<'n, C> {
    fn from(r: SvgElementRender<'n, C>) -> Self {
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
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgElementRender<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgAttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgStaticAttributesOnly<'n, C> {}
impl<'n, C: Component> MethodsForSvgElementContent<'n, C> for SvgStaticAttributes<'n, C> {}

pub struct SvgMatchIfRender<'a, C: Component>(MatchIfRender<'a, C>);

impl<'a, C: Component> SvgMatchIfRender<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> SvgNodesOwned<'a, C> {
        SvgNodesOwned::new(self.0.render_on_arm_index(index))
    }
}
