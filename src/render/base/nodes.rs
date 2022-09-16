use super::{ElementUpdater, ListUpdater};
use crate::{
    component::{Child, ChildComp, Comp, Component},
    dom::{
        AChildNode, ElementStatus, ElementTag, GroupedNodes, Nodes, OwnedComponent, RefComponent,
    },
};
use wasm_bindgen::UnwrapThrowExt;

pub trait NodesRenderMut<C: Component> {
    fn nodes_render_mut(&mut self) -> &mut NodesRender<C>;
}

pub struct NodesRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    update_mode: bool,
    index: usize,
    parent_status: ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut Nodes,
}

impl<'a, C: Component> From<ElementUpdater<'a, C>> for NodesRender<'a, C> {
    fn from(er: ElementUpdater<'a, C>) -> Self {
        let (comp, state, parent_status, element) = er.into_parts();
        let (parent, nodes) = element.ws_node_and_nodes_mut();
        Self {
            comp,
            state,

            update_mode: true,
            index: 0,
            parent_status,
            parent,
            next_sibling: None,
            nodes,
        }
    }
}

impl<'a, C: Component> NodesRender<'a, C> {
    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> Comp<C> {
        self.comp.clone()
    }

    pub fn parent(&self) -> &web_sys::Node {
        self.parent
    }

    pub fn next_sibling(&self) -> Option<&web_sys::Node> {
        self.next_sibling
    }

    pub fn set_static_mode(&mut self) {
        self.update_mode = false;
    }

    pub fn set_update_mode(&mut self) {
        self.update_mode = true;
    }

    pub fn require_render(&self) -> bool {
        if self.update_mode {
            true
        } else {
            self.parent_status == ElementStatus::JustCreated
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

    pub fn get_element_render<E: ElementTag>(&mut self, tag: E) -> ElementUpdater<C> {
        let status = self.nodes.check_or_create_element(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element_mut(self.index);
        // Don't do this here, because .get_element_render() is not always called
        // self.index += 1;
        ElementUpdater::new(self.comp, self.state, element, status)
    }

    pub fn get_match_if_render(&mut self) -> MatchIfRender<C> {
        let grouped_nodes = self
            .nodes
            .grouped_nodes(self.index, self.parent, self.next_sibling);
        self.index += 1;
        MatchIfRender {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            grouped_nodes,
        }
    }

    pub fn get_list_render(&mut self, use_template: bool) -> (&Comp<C>, &C, ListUpdater) {
        let gn = self
            .nodes
            .grouped_nodes(self.index, self.parent, self.next_sibling);
        self.index += 1;
        let (list, next_sibling) = gn.nodes_mut_and_end_flag_node();
        let lr = ListUpdater::new(
            list,
            self.parent,
            self.parent_status,
            Some(next_sibling),
            use_template,
        );
        (self.comp, self.state, lr)
    }

    pub fn component_ref<CC: Component>(&mut self, child: &ChildComp<CC>) {
        // if just created or unmounted:
        // - do first render
        // - attach the root element to self.parent
        // - store the component handle on the node list
        // on the second subsequent renders, do nothing.

        if self.parent_status == ElementStatus::JustCreated || !child.comp_instance().is_mounted() {
            child.first_render();
            child
                .comp_instance()
                .root_element()
                .insert_before_a_sibling(self.parent, self.next_sibling);
            self.nodes
                .store_ref_component(self.index, RefComponent::new(child));
        }
    }

    pub fn component_owned<CC, T>(
        &mut self,
        create_child_comp: impl FnOnce(&C, &Comp<C>) -> Child<C, CC, T>,
    )
    //-> &mut OwnedComponent
    where
        CC: Component,
        T: 'static + Clone + PartialEq,
    {
        let comp = self.comp;
        let state = self.state;
        let ccc = || -> OwnedComponent {
            let cc = create_child_comp(state, comp);
            let root_node = cc.get_root_node();
            OwnedComponent::new(Box::new(cc), root_node)
        };
        let owned_component =
            self.nodes
                .owned_component(self.index, self.parent, self.next_sibling, ccc);
        let just_created = owned_component.just_created();
        let any = owned_component
            .get_any_component_mut()
            .expect_throw("render::base::nodes::NodesRender::component get_any_component");
        match any.downcast_mut::<Child<C, CC, T>>() {
            Some(child) => {
                let have_a_queue_update = child.update(state);
                if just_created && !have_a_queue_update {
                    child.first_render();
                }
            }
            None => log::warn!("Failed to downcast to the expected child component"),
        }
    }

    pub fn new_node(&self) -> bool {
        self.index >= self.nodes.count()
    }

    pub fn nodes_mut(&mut self) -> &mut Nodes {
        self.nodes
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

pub struct MatchIfRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    grouped_nodes: &'a mut GroupedNodes,
}

impl<'a, C: Component> MatchIfRender<'a, C> {
    pub fn new(
        comp: &'a Comp<C>,
        state: &'a C,

        parent: &'a web_sys::Node,
        grouped_nodes: &'a mut GroupedNodes,
    ) -> Self {
        Self {
            comp,
            state,
            parent,
            grouped_nodes,
        }
    }

    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> Comp<C> {
        self.comp.clone()
    }

    pub fn render_on_arm_index(self, index: u32) -> NodesRender<'a, C> {
        let status = self.grouped_nodes.set_active_index(index, self.parent);
        let (nodes, next_sibling) = self.grouped_nodes.nodes_mut_and_end_flag_node();

        NodesRender {
            comp: self.comp,
            state: self.state,

            update_mode: true,
            index: 0,
            parent_status: status,
            parent: self.parent,
            next_sibling: Some(next_sibling),
            nodes,
        }
    }
}
