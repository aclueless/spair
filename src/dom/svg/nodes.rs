pub(crate) struct SvgNodeListUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    index: usize,
    parent_status: crate::dom::ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut crate::dom::NodeList,
}

mod sealed {
    // TODO: Copied from dom::nodes. Should be a common trait?
    pub trait SvgBuilder<C: crate::component::Component> {
        fn require_render(&self) -> bool;
        fn just_created(&self) -> bool;
        fn next_index(&mut self);
        fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C>;
        fn get_match_if_and_increase_index(&mut self) -> super::MatchIfUpdater<C>;
        fn store_raw_wrapper(&mut self, element: crate::dom::Element);
    }
}

pub trait SvgBuilder {
    //
}

