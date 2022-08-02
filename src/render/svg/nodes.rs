use crate::component::{ChildComp, Comp, Component};
use crate::render::base::{ElementRenderMut, MatchIfRender, NodeListRender, NodeListRenderMut};
use super::{SvgElementRender, SvgNameSpace};


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
        clip_path "clipPath"
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
        mask
        mesh
        meshgradient
        meshpatch
        meshrow
        metadata
        mpath
        path
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
