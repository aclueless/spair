use std::any::TypeId;

use super::{ElementUpdater, ListUpdater};
use crate::{
    component::{Child, Comp, Component},
    dom::{
        ComponentRef, ElementStatus, ElementTagExt, GroupedNodes, InternalTextRender, Nodes,
        OwnedComponent,
    },
};
use wasm_bindgen::UnwrapThrowExt;

pub trait NodesUpdaterMut<'n, C: Component> {
    fn nodes_updater_mut(&mut self) -> &mut NodesUpdater<'n, C>;
}

pub struct NodesUpdater<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    update_mode: bool,
    index: usize,
    parent_status: ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut Nodes,
}

impl<'a, C: Component> From<ElementUpdater<'a, C>> for NodesUpdater<'a, C> {
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

impl<'a, C: Component> NodesUpdater<'a, C> {
    pub fn new(
        comp: &'a Comp<C>,
        state: &'a C,
        parent_status: ElementStatus,
        parent: &'a web_sys::Node,
        next_sibling: Option<&'a web_sys::Node>,
        nodes: &'a mut Nodes,
    ) -> Self {
        Self {
            comp,
            state,

            update_mode: true,
            index: 0,
            parent_status,
            parent,
            next_sibling,
            nodes,
        }
    }

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

    pub fn require_update(&self) -> bool {
        if self.update_mode {
            true
        } else {
            self.parent_status == ElementStatus::JustCreated
        }
    }

    pub fn next_index(&mut self) {
        self.index += 1;
    }

    pub fn update_text(&mut self, text: impl InternalTextRender, update_mode: bool) {
        if update_mode || self.parent_status == ElementStatus::JustCreated {
            self.nodes
                .update_text(self.index, text, self.parent, self.next_sibling);
        }
        self.index += 1;
    }

    pub fn get_element_updater<'b, E: ElementTagExt<'b, C>>(
        self: &'b mut NodesUpdater<'a, C>,
        tag: E,
    ) -> Option<E::Updater> {
        let rs = if self.require_update() {
            let status = self.nodes.check_or_create_element(
                tag,
                self.index,
                self.parent_status,
                self.parent,
                self.next_sibling,
            );
            let element = self.nodes.get_element_mut(self.index);
            Some(E::make_updater(ElementUpdater::new(
                self.comp, self.state, element, status,
            )))
        } else {
            None
        };
        self.index += 1;
        rs
    }

    pub fn get_match_if_updater(&mut self) -> MatchIfUpdater<C> {
        let grouped_nodes = self.nodes.grouped_nodes(
            self.index,
            crate::dom::FLAG_NAME_FOR_MATCH_IF,
            self.parent,
            self.next_sibling,
        );
        self.index += 1;
        MatchIfUpdater {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            grouped_nodes,
        }
    }

    pub fn get_list_updater(&mut self, use_template: bool) -> (&Comp<C>, &C, ListUpdater) {
        let gn = self.nodes.grouped_nodes(
            self.index,
            crate::dom::FLAG_NAME_FOR_PARTIAL_LIST,
            self.parent,
            self.next_sibling,
        );
        self.index += 1;
        let (list, next_sibling) = gn.nodes_mut_and_flag_node();
        let lr = ListUpdater::new(
            list,
            self.parent,
            self.parent_status,
            Some(next_sibling),
            use_template,
        );
        (self.comp, self.state, lr)
    }

    pub fn component_ref(&mut self, cr: Box<dyn ComponentRef>) {
        self.nodes
            .ref_component(self.index, cr, self.parent, self.next_sibling);
    }

    pub fn component_owned<CC, T>(
        &mut self,
        create_child_comp: impl FnOnce(&C, &Comp<C>) -> Child<C, CC, T>,
    ) where
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
        if just_created {
            owned_component.set_status_to_existing();
        }
        let any = owned_component
            .get_any_component_mut()
            .expect_throw("render::base::nodes::NodesUpdater::component get_any_component");
        match any.downcast_mut::<Child<C, CC, T>>() {
            Some(child) => {
                let have_a_queue_update = child.update(state);
                if just_created {
                    if have_a_queue_update {
                        child.init();
                    } else {
                        child.first_render();
                    }
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

pub struct MatchIfUpdater<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    grouped_nodes: &'a mut GroupedNodes,
}

impl<'a, C: Component> MatchIfUpdater<'a, C> {
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

    #[doc(hidden)]
    pub fn render_on_arm_index(self, index: TypeId) -> NodesUpdater<'a, C> {
        let status = self.grouped_nodes.set_active_index(index, self.parent);
        let (nodes, next_sibling) = self.grouped_nodes.nodes_mut_and_flag_node();

        NodesUpdater {
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

#[cfg(test)]
mod tests {
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_match_if_same_index_with_different_value_on_the_same_arm() {
        make_a_test_component! {
            type: Option<u32>;
            init: None;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.match_if(|mi| match self.0 {
                    None => crate::set_arm!(mi).done(),
                    Some(value) => crate::set_arm!(mi).update_text(value).done(),
                });
            }
        }

        fn get_grouped_nodes_active_index(nodes: &[crate::dom::Node]) -> Option<std::any::TypeId> {
            let crate::dom::Node::GroupedNodes(mi) = nodes.first().unwrap() else {
                panic!("Expect a GroupNodes for match_if");
            };
            mi.active_index()
        }

        let test = Test::set_up();
        assert_eq!(Some(""), test.text_content().as_deref());

        let active_index_1 = test.execute_on_nodes(get_grouped_nodes_active_index);

        test.update(None);
        assert_eq!(Some(""), test.text_content().as_deref());

        let active_index_2 = test.execute_on_nodes(get_grouped_nodes_active_index);

        // active_index for the same arm must be the same
        assert_eq!(active_index_1, active_index_2);

        test.update(Some(42));
        let active_index_3 = test.execute_on_nodes(get_grouped_nodes_active_index);

        // active_index for different arms must be different
        assert_ne!(active_index_1, active_index_3);
        assert_eq!(Some("42"), test.text_content().as_deref());

        test.update(Some(24));
        let active_index_4 = test.execute_on_nodes(get_grouped_nodes_active_index);

        // active_index for the same arm must be the same
        assert_eq!(active_index_3, active_index_4);
        assert_eq!(Some("24"), test.text_content().as_deref());

        // set to None again
        test.update(None);
        assert_eq!(Some(""), test.text_content().as_deref());

        let active_index_5 = test.execute_on_nodes(get_grouped_nodes_active_index);
        // active_index for the same arm must be the same
        assert_eq!(active_index_1, active_index_5);
    }
}
