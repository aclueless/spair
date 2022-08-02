use crate::component::{Comp, Component};
use crate::dom::{ElementStatus, GroupedNodeList, NameSpace, Nodes};
use crate::render::base::element::ElementRender;

pub trait NodeListRenderMut<C: Component> {
    fn node_list_render_mut(&mut self) -> &mut NodeListRender<C>;
}

pub struct NodeListRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    static_mode: bool,
    index: usize,
    parent_status: ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut Nodes,
}

impl<'a, C: Component> From<ElementRender<'a, C>> for NodeListRender<'a, C> {
    fn from(u: ElementRender<'a, C>) -> Self {
        let (comp, state, status, element) = u.into_parts();
        let (parent, nodes) = element.ws_node_and_node_list_mut();
        Self {
            comp,
            state,

            static_mode: false,
            index: 0,
            parent_status: status,
            parent,
            next_sibling: None,
            nodes,
        }
    }
}

impl<'a, C: Component> NodeListRender<'a, C> {
    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> Comp<C> {
        self.comp.clone()
    }

    pub fn parent(&self) -> &web_sys::Node {
        self.parent
    }

    pub fn set_static_mode(&mut self) {
        self.static_mode = true
    }

    pub fn set_update_mode(&mut self) {
        self.static_mode = false
    }

    pub fn require_render(&self) -> bool {
        if self.static_mode {
            self.parent_status == ElementStatus::JustCreated
        } else {
            true
        }
    }

    pub fn next_index(&mut self) {
        self.index += 1;
    }

    pub fn update_text(&mut self, text: &str) {
        self.nodes
            .update_text(self.index, text, self.parent, self.next_sibling);
        self.index += 1;
    }

    pub fn static_text(&mut self, text: &str) {
        self.nodes
            .static_text(self.index, text, self.parent, self.next_sibling);
        self.index += 1;
    }

    pub fn get_element_render<N: NameSpace>(&mut self, tag: &str) -> ElementRender<C> {
        let status = self.nodes.check_or_create_element::<N>(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element_mut(self.index);
        // Don't do this here, because .get_element_render() is not always called
        // self.index += 1;
        ElementRender::new(self.comp, self.state, element, status)
    }

    pub fn get_match_if_updater(&mut self) -> MatchIfRender<C> {
        let match_if = self
            .nodes
            .grouped_node_list(self.index, self.parent, self.next_sibling);
        self.index += 1;
        MatchIfRender {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            match_if,
        }
    }
}

pub struct MatchIfRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    match_if: &'a mut GroupedNodeList,
}

impl<'a, C: Component> MatchIfRender<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> NodeListRender<'a, C> {
        let status = self.match_if.set_active_index(index, self.parent);
        let (nodes, next_sibling) = self.match_if.nodes_mut_and_end_flag_node();

        NodeListRender {
            comp: self.comp,
            state: self.state,

            static_mode: false,
            index: 0,
            parent_status: status,
            parent: self.parent,
            next_sibling: Some(next_sibling),
            nodes,
        }
    }
}
