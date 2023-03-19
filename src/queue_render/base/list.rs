use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, GroupedNodes, Nodes},
    queue_render::{
        dom::QrListRepresentative,
        vec::{Diff, ListRender},
    },
    render::base::NodesUpdater,
};

type FnNodesUpdater<C, I> = Box<dyn Fn(I, NodesUpdater<C>)>;

pub struct QrListRender<C: Component, I> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: Nodes,
    end_flag_node: Option<web_sys::Node>,
    use_template: bool,
    fn_render: FnNodesUpdater<C, I>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, I: Clone> ListRender<I> for QrListRender<C, I> {
    fn render(&mut self, items: &[I], diffs: Vec<Diff<I>>) {
        self.render_list(items, diffs);
    }

    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component, I: Clone> QrListRender<C, I> {
    pub fn new(
        comp: Comp<C>,
        parent: web_sys::Node,
        end_flag_node: Option<web_sys::Node>,
        fn_render: impl Fn(I, NodesUpdater<C>) + 'static,
        use_template: bool,
    ) -> Self {
        Self {
            comp,
            parent,
            nodes: Nodes::default(),
            end_flag_node,
            use_template,
            fn_render: Box::new(fn_render),
            unmounted: Rc::new(Cell::new(false)),
        }
    }
    pub fn make_representative(&self) -> QrListRepresentative {
        QrListRepresentative::new(self.end_flag_node.clone(), self.unmounted.clone())
    }

    pub fn render_list(&mut self, items: &[I], diffs: Vec<Diff<I>>) {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrListRender::render::rc_comp.try_borrow().");
        let state = comp.state();
        if diffs.iter().any(|d| matches!(d, Diff::New)) {
            self.render_change(state, items, Diff::New);
        } else {
            for d in diffs {
                self.render_change(state, items, d);
            }
        }
    }

    fn render_change(&mut self, state: &C, items: &[I], diff: Diff<I>) {
        match diff {
            Diff::New => self.all_new(state, items.to_vec()),
            Diff::Push { value } => self.push(state, value),
            Diff::Pop => self.pop(),
            Diff::Insert { index, value } => self.insert(state, index, value),
            Diff::RemoveAt { index } => self.remove(index),
            Diff::ReplaceAt { index, new_value } => self.re_render(state, index, new_value),
            Diff::Move {
                old_index,
                new_index,
            } => self.move_item(old_index, new_index),
            Diff::Swap { index_1, index_2 } => self.swap(index_1, index_2),
            Diff::Render { index, value } => self.re_render(state, index, value),
        }
    }

    fn clear(&mut self) {
        if self.end_flag_node.is_none() {
            self.parent.set_text_content(None);
            self.nodes.clear_vec();
        } else {
            self.nodes.clear_and_remove_child_from_dom(&self.parent);
        }
    }

    fn all_new(&mut self, state: &C, items: Vec<I>) {
        self.clear();
        for item in items {
            self.push(state, item);
        }
    }

    fn push(&mut self, state: &C, item: I) {
        let index = self.nodes.count();
        let (status, group, next_sibling) = self.nodes.recipe_for_list_entry(
            index,
            &self.parent,
            ElementStatus::JustCreated,
            self.end_flag_node.as_ref(),
            self.use_template,
        );
        let u = NodesUpdater::new(
            &self.comp,
            state,
            status,
            &self.parent,
            next_sibling.as_ref().or(self.end_flag_node.as_ref()),
            group.nodes_mut(),
        );
        (self.fn_render)(item, u);
    }

    fn pop(&mut self) {
        if let Some(group) = self.nodes.pop_group() {
            group.remove_from_dom(&self.parent);
        }
    }

    fn insert(&mut self, state: &C, index: usize, item: I) {
        // An insert at the end of the list is handled by QrVec as a push
        let existing_group = self.nodes.get_grouped_nodes(index);
        let next_sibling = existing_group.map(|g| g.flag_node_ref());
        let (mut new_group, status) = if self.use_template {
            let new_group = existing_group
                .expect_throw("guanrantee valid index by QrVec::insert")
                .clone_list_entry();
            (new_group, ElementStatus::JustCloned)
        } else {
            (
                GroupedNodes::with_flag_name(crate::dom::FLAG_NAME_FOR_LIST_ENTRY),
                ElementStatus::JustCreated,
            )
        };
        new_group.insert_before_a_sibling(&self.parent, next_sibling);

        let u = NodesUpdater::new(
            &self.comp,
            state,
            status,
            &self.parent,
            next_sibling,
            new_group.nodes_mut(),
        );
        (self.fn_render)(item, u);
        self.nodes.insert_group_at(index, new_group);
    }

    fn remove(&mut self, index: usize) {
        let group = self.nodes.remove_group_at(index);
        group.remove_from_dom(&self.parent);
    }

    fn re_render(&mut self, state: &C, index: usize, item: I) {
        let next_sibling = self
            .nodes
            .get_grouped_nodes(index + 1)
            .map(|g| g.flag_node_ref().clone());
        let group = self.nodes.get_grouped_nodes_mut(index);
        let u = NodesUpdater::new(
            &self.comp,
            state,
            ElementStatus::Existing,
            &self.parent,
            next_sibling.as_ref().or(self.end_flag_node.as_ref()),
            group.nodes_mut(),
        );
        (self.fn_render)(item, u);
    }

    fn move_item(&mut self, old_index: usize, new_index: usize) {
        let group = self.nodes.remove_group_at(old_index);
        let next_sibling = self
            .nodes
            .get_grouped_nodes(new_index)
            .map(|g| g.flag_node_ref())
            .or(self.end_flag_node.as_ref());
        group.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_group_at(new_index, group);
    }

    fn swap(&mut self, index_1: usize, index_2: usize) {
        let (low_index, high_index) = if index_1 < index_2 {
            (index_1, index_2)
        } else {
            (index_2, index_1)
        };
        let high_group = self.nodes.remove_group_at(high_index);
        let low_group = self.nodes.remove_group_at(low_index);

        // low_node is not removed from DOM yet
        let low_node = low_group.flag_node_ref();
        high_group.insert_before_a_sibling(&self.parent, Some(low_node));
        self.nodes.insert_group_at(low_index, high_group);

        let next_sibling = self
            .nodes
            .get_grouped_nodes(high_index)
            .map(|g| g.flag_node_ref())
            .or(self.end_flag_node.as_ref());

        low_group.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_group_at(high_index, low_group);
    }
}

#[cfg(test)]
mod qr_list_tests {
    use crate::queue_render::vec::QrVec;

    macro_rules! make_queue_render_list_test {
        ($mode:expr) => {
            make_a_test_component! {
                type: QrVec<u32>;
                init: QrVec::with_values(Vec::new());
                render_fn: fn render(&self, element: crate::Element<Self>) {
                    element.qr_list(&self.0, $mode, render_u32);
                }
            }

            fn render_u32(value: u32, nodes: crate::Nodes<TestComponent>) {
                nodes.update_text(value);
            }

            let test = Test::set_up();
            assert_eq!(Some(""), test.text_content().as_deref());

            test.update_with(|qr| qr.get_mut().new_values(vec![1, 5, 3, 7]));
            assert_eq!(Some("1537"), test.text_content().as_deref());

            test.update_with(|qr| qr.get_mut().push(4));
            assert_eq!(Some("15374"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().remove_at(1);
            });
            assert_eq!(Some("1374"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().pop();
            });
            assert_eq!(Some("137"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().insert_at(0, 2).unwrap();
            });
            assert_eq!(Some("2137"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().insert_at(1, 8).unwrap();
            });
            assert_eq!(Some("28137"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().insert_at(5, 5).unwrap();
            });
            assert_eq!(Some("281375"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().swap(0, 5).unwrap();
            });
            assert_eq!(Some("581372"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().swap(4, 0).unwrap();
            });
            assert_eq!(Some("781352"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().swap(2, 5).unwrap();
            });
            assert_eq!(Some("782351"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().r#move(1, 5).unwrap();
            });
            assert_eq!(Some("723518"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().r#move(0, 3).unwrap();
            });
            assert_eq!(Some("235718"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().r#move(3, 0).unwrap();
            });
            assert_eq!(Some("723518"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().r#move(5, 0).unwrap();
            });
            assert_eq!(Some("872351"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().replace_at(2, 0);
            });
            assert_eq!(Some("870351"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().replace_at(5, 9);
            });
            assert_eq!(Some("870359"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().replace_at(0, 6);
            });
            assert_eq!(Some("670359"), test.text_content().as_deref());

            test.update_with(|qr| {
                let mut qr = qr.get_mut();
                qr.push(0);
                qr.new_values(vec![1, 2, 3, 4, 5, 6]);
            });
            assert_eq!(Some("123456"), test.text_content().as_deref());

            test.update_with(|qr| {
                qr.get_mut().clear();
            });
            assert_eq!(Some(""), test.text_content().as_deref());
        };
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn qr_list_clone() {
        make_queue_render_list_test!(crate::ListElementCreation::Clone);
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn qr_list_new() {
        make_queue_render_list_test!(crate::ListElementCreation::New);
    }
}
