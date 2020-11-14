pub(crate) struct SvgNodeListUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    index: usize,
    parent_status: crate::dom::ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut crate::dom::NodeList,
}

impl<'a, C: crate::component::Component> SvgNodeListUpdater<'a, C> {
    pub fn from_svg_updater(eu: super::SvgUpdater<'a, C>) -> Self {
        Self {
            comp: eu.comp,
            state: eu.state,
            index: 0,
            parent_status: eu.status,
            parent: eu.element.ws_element.as_ref(),
            next_sibling: None,
            nodes: &mut eu.element.nodes,
        }
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::SvgUpdater<C> {
        let status = self.nodes.check_or_create_element(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element(self.index);
        self.index += 1;
        super::SvgUpdater::new(self.comp, self.state, element, status)
    }

    // fn get_match_if_updater(&mut self) -> MatchIfUpdater<C> {
    //     let match_if = self
    //         .nodes
    //         .fragmented_node_list(self.index, self.parent, self.next_sibling);
    //     self.index += 1;
    //     MatchIfUpdater {
    //         comp: self.comp,
    //         state: self.state,
    //         parent: self.parent,
    //         match_if,
    //     }
    // }

    // #[cfg(feature = "partial-non-keyed-list")]
    // pub fn list_with_render<I, R>(
    //     &mut self,
    //     items: impl IntoIterator<Item = I>,
    //     mode: super::ListElementCreation,
    //     tag: &str,
    //     render: R,
    // ) where
    //     for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    // {
    //     let use_template = mode.use_template();
    //     let fragmented_node_list =
    //         self.nodes
    //             .fragmented_node_list(self.index, self.parent, self.next_sibling);
    //     self.index += 1;
    //     let mut non_keyed_list_updater = super::NonKeyedListUpdater::new(
    //         self.comp,
    //         self.state,
    //         &mut fragmented_node_list.nodes,
    //         tag,
    //         self.parent,
    //         Some(&fragmented_node_list.end_node),
    //         use_template,
    //     );
    //     let _select_element_value_will_be_set_on_dropping =
    //         non_keyed_list_updater.update(items, render);
    // }
}

mod sealed {
    // TODO: Copied from dom::nodes. Should be a common trait?
    pub trait SvgBuilder<C: crate::component::Component> {
        fn require_render(&self) -> bool;
        fn just_created(&self) -> bool;
        fn next_index(&mut self);
        fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::SvgUpdater<C>;
        //fn get_match_if_and_increase_index(&mut self) -> super::SvgMatchIfUpdater<C>;
        //fn store_raw_wrapper(&mut self, element: crate::dom::Element);
    }
}

macro_rules! create_methods_for_tags {
    ($($tag:ident)+) => {
        $(
            fn $tag(self, f: impl FnOnce(super::SvgUpdater<C>)) -> Self::Output {
                self.render_element(stringify!($tag), f)
            }
        )+
    }
}

pub trait SvgBuilder<C: crate::component::Component>: Sized {
    type Output: From<Self> + sealed::SvgBuilder<C>;

    // fn match_if(self, f: impl FnOnce(MatchIfUpdater<C>)) -> Self::Output {
    //     use sealed::SvgBuilder;
    //     let mut this: Self::Output = self.into();
    //     f(this.get_match_if_and_increase_index());
    //     this
    // }

    fn render_element(self, tag: &str, f: impl FnOnce(super::SvgUpdater<C>)) -> Self::Output {
        use sealed::SvgBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            f(this.get_element_and_increase_index(tag));
        } else {
            this.next_index();
        }
        this
    }

    create_methods_for_tags! {
        circle
    }

    fn svg(self, f: impl FnOnce(super::SvgUpdater<C>)) -> Self::Output {
        use sealed::SvgBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            f(this.get_element_and_increase_index("svg"));
        } else {
            this.next_index();
        }
        this
    }
}

pub struct SvgStaticNodesOwned<'a, C: crate::component::Component>(SvgNodeListUpdater<'a, C>);

impl<'a, C: crate::component::Component> SvgStaticNodesOwned<'a, C> {
    pub(super) fn from_svg_updater(su: super::SvgUpdater<'a, C>) -> Self {
        Self(SvgNodeListUpdater::from_svg_updater(su))
    }

    pub fn state(&self) -> &'a C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    // pub fn nodes(self) -> NodesOwned<'a, C> {
    //     SvgNodesOwned(self.0)
    // }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    // pub fn render(mut self, value: impl crate::renderable::Render<C>) -> Self {
    //     let nodes = SvgNodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    // pub fn render_ref(mut self, value: &impl crate::renderable::RenderRef<C>) -> Self {
    //     let nodes = Nodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    // pub fn r#static(mut self, value: impl crate::renderable::StaticRender<C>) -> Self {
    //     let static_nodes = StaticNodes(&mut self.0);
    //     value.render(static_nodes);
    //     self
    // }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::renderable::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.parent_status != super::ElementStatus::Existing {
    //         value.render(self.nodes()).static_nodes()
    //     } else {
    //         self.0.index += 1;
    //         self
    //     }
    // }
}

pub struct SvgNodesOwned<'a, C: crate::component::Component>(SvgNodeListUpdater<'a, C>);

impl<'a, C: crate::component::Component> SvgNodesOwned<'a, C> {
    pub(super) fn from_svg_updater(su: super::SvgUpdater<'a, C>) -> Self {
        Self(SvgNodeListUpdater::from_svg_updater(su))
    }

    // pub(super) fn nodes_ref<'n>(&'n mut self) -> Nodes<'n, 'a, C> {
    //     Nodes(&mut self.0)
    // }

    // pub(super) fn static_nodes_ref<'n>(&'n mut self) -> StaticNodes<'n, 'a, C> {
    //     StaticNodes(&mut self.0)
    // }

    pub fn state(&self) -> &'a C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    pub fn static_nodes(self) -> SvgStaticNodesOwned<'a, C> {
        SvgStaticNodesOwned(self.0)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    // pub fn render(mut self, value: impl crate::renderable::Render<C>) -> Self {
    //     let nodes = Nodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    // pub fn render_ref(mut self, value: &impl crate::renderable::RenderRef<C>) -> Self {
    //     let nodes = Nodes(&mut self.0);
    //     value.render(nodes);
    //     self
    // }

    // pub fn r#static(mut self, value: impl crate::renderable::StaticRender<C>) -> Self {
    //     let static_nodes = StaticNodes(&mut self.0);
    //     value.render(static_nodes);
    //     self
    // }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::renderable::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.parent_status != super::ElementStatus::Existing {
    //         value.render(self)
    //     } else {
    //         self.0.index += 1;
    //         self
    //     }
    // }

    // pub(crate) fn update_text(mut self, text: &str) -> Self {
    //     self.0
    //         .nodes
    //         .update_text(self.0.index, text, self.0.parent, self.0.next_sibling);
    //     self.0.index += 1;
    //     self
    // }

    // #[cfg(feature = "partial-non-keyed-list")]
    // pub fn list_with_render<I, R>(
    //     mut self,
    //     items: impl IntoIterator<Item = I>,
    //     mode: super::ListElementCreation,
    //     tag: &str,
    //     render: R,
    // ) -> Self
    // where
    //     for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    // {
    //     self.0.list_with_render(items, mode, tag, render);
    //     self
    // }
}
