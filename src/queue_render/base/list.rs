use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{AChildNode, Element, ElementStatus, ElementTag, Nodes},
    queue_render::{
        dom::QrListRepresentative,
        vec::{Diff, ListRender},
    },
    render::base::ElementUpdater,
};

pub struct QrListRender<C: Component, E, I> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: Nodes,
    end_flag_node: Option<web_sys::Node>,
    element_tag: E,
    use_template: bool,
    fn_render: Box<dyn Fn(I, ElementUpdater<C>)>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, E: ElementTag, I: Clone> ListRender<I> for QrListRender<C, E, I> {
    fn render(&mut self, items: &[I], diffs: Vec<Diff<I>>) {
        self.render_list(items, diffs);
    }

    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component, E: ElementTag, I: Clone> QrListRender<C, E, I> {
    pub fn new(
        element_tag: E,
        comp: Comp<C>,
        parent: web_sys::Node,
        end_flag_node: Option<web_sys::Node>,
        fn_render: impl Fn(I, ElementUpdater<C>) + 'static,
        use_template: bool,
    ) -> Self {
        Self {
            comp,
            parent,
            nodes: Nodes::default(),
            end_flag_node,
            element_tag,
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
            Diff::Push(item) => self.push(state, item),
            Diff::Pop => self.pop(),
            Diff::Insert { index, value } => self.insert(state, index, value),
            Diff::RemoveAtIndex(index) => self.remove(index),
            Diff::ReplaceAt { index, new_value } => self.re_render(state, index, new_value),
            Diff::Move {
                old_index,
                new_index,
            } => self.move_item(old_index, new_index),
            Diff::Swap { index_1, index_2 } => self.swap(index_1, index_2),
            Diff::Clear => self.clear(),
        }
    }

    fn clear(&mut self) {
        if self.end_flag_node.is_none() {
            self.parent.set_text_content(None);
            self.nodes.clear_vec();
        } else {
            self.nodes.clear(&self.parent);
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
        let status = self.nodes.check_or_create_element_for_list(
            self.element_tag,
            index,
            &self.parent,
            ElementStatus::JustCreated,
            self.end_flag_node.as_ref(),
            self.use_template,
        );
        let element = self.nodes.get_element_mut(index);
        let render = ElementUpdater::new(&self.comp, state, element, status);
        (self.fn_render)(item, render);
    }

    fn pop(&mut self) {
        if let Some(element) = self.nodes.pop_element() {
            element.remove_from(&self.parent);
        }
    }

    fn insert(&mut self, state: &C, index: usize, item: I) {
        // An insert at the end of the list is handled by QrVec as a push
        let existing_element = self.nodes.get_element(index);
        let next_sibling = existing_element.map(|e| e.ws_node());
        let (mut new_element, status) = if self.use_template {
            let new_element = existing_element
                .expect_throw("guanrantee valid index by QrVec::insert")
                .clone();
            (new_element, ElementStatus::JustCloned)
        } else {
            let element = Element::new_ns(self.element_tag);
            (element, ElementStatus::JustCreated)
        };
        let render = ElementUpdater::new(&self.comp, state, &mut new_element, status);
        (self.fn_render)(item, render);
        new_element.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_element_at(index, new_element);
    }

    fn remove(&mut self, index: usize) {
        let element = self.nodes.remove_element_at(index);
        element.remove_from(&self.parent);
    }

    fn re_render(&mut self, state: &C, index: usize, item: I) {
        let element = self.nodes.get_element_mut(index);
        let render = ElementUpdater::new(&self.comp, state, element, ElementStatus::Existing);
        (self.fn_render)(item, render);
    }

    fn move_item(&mut self, old_index: usize, new_index: usize) {
        let element = self.nodes.remove_element_at(old_index);
        let next_sibling = self
            .nodes
            .get_element(new_index)
            .map(|e| e.ws_node())
            .or(self.end_flag_node.as_ref());
        element.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_element_at(new_index, element);
    }

    fn swap(&mut self, index_1: usize, index_2: usize) {
        let (low_index, high_index) = if index_1 < index_2 {
            (index_1, index_2)
        } else {
            (index_2, index_1)
        };
        let high_element = self.nodes.remove_element_at(high_index);
        let low_element = self.nodes.remove_element_at(low_index);

        let low_node = low_element.ws_node();
        high_element.insert_before_a_sibling(&self.parent, Some(low_node));
        self.nodes.insert_element_at(low_index, high_element);

        let next_sibling = self
            .nodes
            .get_element(high_index)
            .map(|e| e.ws_node())
            .or(self.end_flag_node.as_ref());

        low_element.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_element_at(high_index, low_element);
    }
}

#[cfg(test)]
mod qr_list_tests {
    use wasm_bindgen_test::*;

    use crate::prelude::*;

    use crate::component::{Component, RcComp};
    use crate::dom::{Element, Node};
    use crate::queue_render::vec::QrVec;
    use crate::render::html::ElementRender;
    use crate::render::ListElementCreation;

    pub struct State {
        vec: QrVec<u32>,
    }

    impl ElementRender<State> for u32 {
        const ELEMENT_TAG: &'static str = "span";
        fn render(self, item: crate::Element<State>) {
            item.rupdate(self);
        }
    }

    impl Component for State {
        type Routes = ();
        fn render(&self, element: crate::Element<Self>) {
            element
                .div(|d| {
                    d.id("qr_clone")
                        .qr_list(&self.vec, ListElementCreation::Clone);
                })
                .div(|d| {
                    d.id("qr_new").qr_list(&self.vec, ListElementCreation::New);
                });
        }
    }

    impl Application for State {
        fn init(_comp: &crate::Comp<Self>) -> Self {
            Self {
                vec: QrVec::with_values(vec![1, 5, 3, 7]),
            }
        }
    }

    fn get_text(node: Option<&Node>) -> Option<String> {
        match node.expect_throw("Should not empty") {
            Node::Element(e) => {
                return e.ws_element().ws_node().text_content();
            }
            n => panic!("Expected an Node::Element, found {:?}", n),
        }
    }

    fn qr_list_test(
        rc: &RcComp<State>,
        do_change: impl FnOnce(&QrVec<u32>),
    ) -> (Option<String>, Option<String>) {
        do_change(&rc.comp_instance().state().vec);
        crate::queue_render::execute_render_queue();
        let comp_instance = rc.comp_instance();
        let root = comp_instance.root_element();
        assert_eq!(2, root.nodes().count());

        let render_clone = get_text(root.nodes().nodes_vec().get(0));
        let render_new = get_text(root.nodes().nodes_vec().get(1));
        (render_clone, render_new)
    }

    macro_rules! both_eq {
        ($x:literal, $expr:expr) => {
            let r = $expr;
            assert_eq!(Some($x), r.0.as_deref(), "by cloning");
            assert_eq!(Some($x), r.1.as_deref(), "by creating new");
        };
    }

    #[wasm_bindgen_test]
    fn qr_list() {
        let root = Element::new_ns(crate::render::html::HtmlTag("div"));
        let rc =
            crate::application::mount_to_element::<State>(root.ws_element().clone().into_inner());

        both_eq! { "1537", qr_list_test(&rc, |_| {}) }
        both_eq! { "15374", qr_list_test(&rc, |vec| vec.push(4)) }
        both_eq! { "1374", qr_list_test(&rc, |vec| { vec.remove_at(1); }) }
        both_eq! { "137", qr_list_test(&rc, |vec| { vec.pop(); }) }
        both_eq! { "2137", qr_list_test(&rc, |vec| { vec.insert_at(0, 2).expect_throw("insert at 0"); }) }
        both_eq! { "28137", qr_list_test(&rc, |vec| { vec.insert_at(1, 8).expect_throw("insert at 1"); }) }
        both_eq! { "281375", qr_list_test(&rc, |vec| { vec.insert_at(5, 5).expect_throw("insert at 5"); }) }
        both_eq! { "581372", qr_list_test(&rc, |vec| { vec.swap(0, 5).expect_throw("swap 0-5"); }) }
        both_eq! { "781352", qr_list_test(&rc, |vec| { vec.swap(4, 0).expect_throw("swap 0-4"); }) }
        both_eq! { "782351", qr_list_test(&rc, |vec| { vec.swap(2, 5).expect_throw("swap 2-5"); }) }
        both_eq! { "723518", qr_list_test(&rc, |vec| { vec.r#move(1, 5).expect_throw("move 1-5"); }) }
        both_eq! { "235718", qr_list_test(&rc, |vec| { vec.r#move(0, 3).expect_throw("move 0-3"); }) }
        both_eq! { "723518", qr_list_test(&rc, |vec| { vec.r#move(3, 0).expect_throw("move 3-0"); }) }
        both_eq! { "872351", qr_list_test(&rc, |vec| { vec.r#move(5, 0).expect_throw("move 5-0"); }) }
        both_eq! { "870351", qr_list_test(&rc, |vec| { vec.replace_at(2, 0).expect_throw("move at 2"); }) }
        both_eq! { "870359", qr_list_test(&rc, |vec| { vec.replace_at(5, 9).expect_throw("move at 5"); }) }
        both_eq! { "670359", qr_list_test(&rc, |vec| { vec.replace_at(0, 6).expect_throw("move at 0"); }) }
        both_eq! { "123456", qr_list_test(&rc, |vec| { vec.push(0); vec.new_values(vec![1,2,3,4,5,6]); }) }
        both_eq! { "", qr_list_test(&rc, |vec| { vec.clear(); }) }
    }
}
