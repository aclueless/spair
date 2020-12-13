macro_rules! create_methods_for_tags {
    ($($fn_name:ident $($tag:literal)?)+) => {
        $(
            create_methods_for_tags!(@single $fn_name $($tag)?);
        )+
    };
    (@single $fn_name:ident) => {
        create_methods_for_tags!(@impl $fn_name stringify!($fn_name));
    };
    (@single $fn_name:ident $tag:literal) => {
        create_methods_for_tags!(@impl $fn_name $tag);
    };
    (@impl $fn_name:ident $tag:expr) => {
        fn $fn_name(self, f: impl FnOnce(super::SvgUpdater<C>)) -> Self::Output {
            self.render_element($tag, f)
        }
    }
}

pub trait SvgBuilder<C: crate::component::Component>: Sized {
    type Output: From<Self> + crate::dom::nodes::DomBuilder<C>;

    // fn match_if(self, f: impl FnOnce(MatchIfUpdater<C>)) -> Self::Output {
    //     use sealed::SvgBuilder;
    //     let mut this: Self::Output = self.into();
    //     f(this.get_match_if_and_increase_index());
    //     this
    // }

    fn render_element(self, tag: &str, f: impl FnOnce(super::SvgUpdater<C>)) -> Self::Output {
        use crate::dom::nodes::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            f(this.get_element_and_increase_index(tag).into());
        } else {
            this.next_index();
        }
        this
    }

    // https://developer.mozilla.org/en-US/docs/Web/SVG/Element
    create_methods_for_tags! {
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
}

pub struct SvgStaticNodesOwned<'a, C>(crate::dom::nodes::NodeListUpdater<'a, C>);
pub struct SvgNodesOwned<'a, C>(crate::dom::nodes::NodeListUpdater<'a, C>);
pub struct SvgStaticNodes<'n, 'h: 'n, C>(&'n mut crate::dom::nodes::NodeListUpdater<'h, C>);
pub struct SvgNodes<'n, 'h: 'n, C>(&'n mut crate::dom::nodes::NodeListUpdater<'h, C>);

impl<'a, C> From<crate::dom::HtmlUpdater<'a, C>> for SvgStaticNodesOwned<'a, C> {
    fn from(u: crate::dom::HtmlUpdater<'a, C>) -> Self {
        Self(u.u.into())
    }
}

impl<'a, C> From<crate::dom::HtmlUpdater<'a, C>> for SvgNodesOwned<'a, C> {
    fn from(u: crate::dom::HtmlUpdater<'a, C>) -> Self {
        Self(u.u.into())
    }
}

impl<'a, C> From<crate::dom::ElementUpdater<'a, C>> for SvgStaticNodesOwned<'a, C> {
    fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self {
        Self(u.into())
    }
}

impl<'a, C> From<crate::dom::ElementUpdater<'a, C>> for SvgNodesOwned<'a, C> {
    fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self {
        Self(u.into())
    }
}

impl<'a, C: crate::component::Component> From<crate::dom::SvgStaticAttributes<'a, C>>
    for SvgNodesOwned<'a, C>
{
    fn from(sa: crate::dom::SvgStaticAttributes<'a, C>) -> Self {
        sa.nodes()
    }
}

impl<'a, C: crate::component::Component> SvgStaticNodesOwned<'a, C> {
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

    pub fn nodes(self) -> SvgNodesOwned<'a, C> {
        SvgNodesOwned(self.0)
    }

    pub fn render(mut self, value: impl crate::dom::SvgRender<C>) -> Self {
        let nodes = SvgNodes(&mut self.0);
        value.render(nodes);
        self
    }

    // pub fn render_ref(mut self, value: &impl crate::dom::SvgRenderRef<C>) -> Self {
    //     let nodes = SvgNodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    pub fn r#static(mut self, value: impl crate::dom::SvgStaticRender<C>) -> Self {
        let static_nodes = SvgStaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }
}

impl<'a, C: crate::component::Component> SvgNodesOwned<'a, C> {
    pub(in crate::dom) fn nodes_ref<'n>(&'n mut self) -> SvgNodes<'n, 'a, C> {
        SvgNodes(&mut self.0)
    }

    pub(in crate::dom) fn static_nodes_ref<'n>(&'n mut self) -> SvgStaticNodes<'n, 'a, C> {
        SvgStaticNodes(&mut self.0)
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

    pub fn static_nodes(self) -> SvgStaticNodesOwned<'a, C> {
        SvgStaticNodesOwned(self.0)
    }

    pub fn render(mut self, value: impl crate::dom::SvgRender<C>) -> Self {
        let nodes = SvgNodes(&mut self.0);
        value.render(nodes);
        self
    }

    // pub fn render_ref(mut self, value: &impl crate::dom::SvgRenderRef<C>) -> Self {
    //     let nodes = SvgNodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    pub fn r#static(mut self, value: impl crate::dom::SvgStaticRender<C>) -> Self {
        let static_nodes = SvgStaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list<I>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
    ) -> Self
    where
        I: Copy,
        I: crate::dom::SvgListItemRender<C>,
    {
        self.0
            .svg_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
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
        for<'u> R: Fn(I, crate::dom::SvgUpdater<'u, C>),
    {
        self.0.svg_list_with_render(items, mode, tag, render);
        self
    }
}

impl<'n, 'h, C: crate::component::Component> SvgStaticNodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp()
    }

    pub fn nodes(self) -> SvgNodes<'n, 'h, C> {
        SvgNodes(self.0)
    }

    pub fn render(self, value: impl crate::dom::SvgRender<C>) -> Self {
        let nodes = SvgNodes(self.0);
        value.render(nodes);
        self
    }

    // pub fn render_ref(self, value: &impl crate::dom::SvgRenderRef<C>) -> Self {
    //     let nodes = SvgNodes(self.0);
    //     value.render(nodes);
    //     self
    // }

    pub fn r#static(self, value: impl crate::dom::SvgStaticRender<C>) -> Self {
        let static_nodes = SvgStaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    pub(crate) fn static_text(self, text: &str) -> Self {
        self.0.static_text(text);
        self
    }
}

impl<'n, 'h, C: crate::component::Component> SvgNodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.state()
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp()
    }

    pub fn static_nodes(self) -> SvgStaticNodes<'n, 'h, C> {
        SvgStaticNodes(self.0)
    }

    pub fn render(self, value: impl crate::dom::SvgRender<C>) -> Self {
        let nodes = SvgNodes(self.0);
        value.render(nodes);
        self
    }

    // pub fn render_ref(self, value: &impl crate::dom::SvgRenderRef<C>) -> Self {
    //     let nodes = SvgNodes(self.0);
    //     value.render(nodes);
    //     self
    // }

    pub fn r#static(self, value: impl crate::dom::SvgStaticRender<C>) -> Self {
        let static_nodes = SvgStaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    pub(crate) fn update_text(self, text: &str) -> Self {
        self.0.update_text(text);
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list<I>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
    ) -> Self
    where
        I: Copy,
        I: crate::dom::SvgListItemRender<C>,
    {
        self.0
            .svg_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
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
        for<'u> R: Fn(I, crate::dom::SvgUpdater<'u, C>),
    {
        self.0.svg_list_with_render(items, mode, tag, render);
        self
    }
}

impl<'a, C: crate::component::Component> crate::dom::nodes::DomBuilder<C>
    for SvgStaticNodesOwned<'a, C>
{
    fn require_render(&self) -> bool {
        self.0.parent_status() == crate::dom::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.just_created()
    }

    fn next_index(&mut self) {
        self.0.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.get_svg_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.store_raw_wrapper(element);
    }
}

impl<'a, C: crate::component::Component> crate::dom::nodes::DomBuilder<C> for SvgNodesOwned<'a, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.just_created()
    }

    fn next_index(&mut self) {
        self.0.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.get_svg_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.store_raw_wrapper(element);
    }
}

impl<'n, 'h, C: crate::component::Component> crate::dom::nodes::DomBuilder<C>
    for SvgStaticNodes<'n, 'h, C>
{
    fn require_render(&self) -> bool {
        self.0.parent_status() == crate::dom::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.just_created()
    }

    fn next_index(&mut self) {
        self.0.next_index()
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.get_svg_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.store_raw_wrapper(element);
    }
}

impl<'n, 'h, C: crate::component::Component> crate::dom::nodes::DomBuilder<C>
    for SvgNodes<'n, 'h, C>
{
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.just_created()
    }

    fn next_index(&mut self) {
        self.0.next_index();
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C> {
        self.0.get_svg_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> crate::dom::nodes::MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: crate::dom::Element) {
        self.0.store_raw_wrapper(element);
    }
}

impl<'a, C: crate::component::Component> SvgBuilder<C> for SvgStaticNodesOwned<'a, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> SvgBuilder<C> for SvgNodesOwned<'a, C> {
    type Output = Self;
}

impl<'n, 'h, C: crate::component::Component> SvgBuilder<C> for SvgStaticNodes<'n, 'h, C> {
    type Output = Self;
}

impl<'n, 'h, C: crate::component::Component> SvgBuilder<C> for SvgNodes<'n, 'h, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> SvgBuilder<C> for crate::dom::SvgStaticAttributes<'a, C> {
    type Output = SvgNodesOwned<'a, C>;
}
