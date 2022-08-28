use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{AChildNode, Element, ElementStatus, ElementTag, Nodes},
    queue_render::{
        dom::QrListRepresentative,
        vec::{Diff, ListRender},
    },
    render::base::ElementRender,
};

pub struct QrListRender<C: Component, E, I> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: Nodes,
    end_flag_node: Option<web_sys::Node>,
    element_tag: E,
    use_template: bool,
    fn_render: Box<dyn Fn(&I, ElementRender<C>)>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, E: ElementTag, I> ListRender<I> for QrListRender<C, E, I> {
    fn render(&mut self, items: &[I], diffs: &[Diff<I>]) {
        self.render_list(items, diffs);
    }

    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component, E: ElementTag, I> QrListRender<C, E, I> {
    pub fn new(
        element_tag: E,
        comp: Comp<C>,
        parent: web_sys::Node,
        end_flag_node: Option<web_sys::Node>,
        fn_render: impl Fn(&I, ElementRender<C>) + 'static,
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

    pub fn render_list(&mut self, items: &[I], diffs: &[Diff<I>]) {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrListRender::render::rc_comp.try_borrow().");
        let state = comp.state();
        if diffs.iter().any(|d| matches!(d, Diff::New)) {
            self.render_change(state, items, &Diff::New);
        } else {
            for d in diffs.iter() {
                self.render_change(state, items, d);
            }
        }
    }

    fn render_change(&mut self, state: &C, items: &[I], diff: &Diff<I>) {
        match diff {
            Diff::New => self.all_new(state, items),
            Diff::Push(item) => self.push(state, item),
            Diff::Pop => self.pop(),
            Diff::Insert { index, value } => self.insert(state, *index, value),
            Diff::RemoveAtIndex(index) => self.remove(*index),
            Diff::ReplaceAt { index, new_value } => self.re_render(state, *index, new_value),
            Diff::Move {
                old_index,
                new_index,
            } => self.move_item(*old_index, *new_index),
            Diff::Swap { index_1, index_2 } => self.swap(*index_1, *index_2),
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

    fn all_new(&mut self, state: &C, items: &[I]) {
        self.clear();
        for item in items {
            self.push(state, item);
        }
    }

    fn push(&mut self, state: &C, item: &I) {
        let index = self.nodes.count();
        let status = self.nodes.check_or_create_element_for_list(
            self.element_tag,
            index,
            &self.parent,
            self.end_flag_node.as_ref(),
            self.use_template,
        );
        let element = self.nodes.get_element_mut(index);
        let render = ElementRender::new(&self.comp, state, element, status);
        (self.fn_render)(item, render);
    }

    fn pop(&mut self) {
        if let Some(element) = self.nodes.pop_element() {
            element.remove_from(&self.parent);
        }
    }

    fn insert(&mut self, state: &C, index: usize, item: &I) {
        let (mut new_element, status, next_sibling) = if self.use_template {
            let existing_element = self.nodes.get_element_mut(index);
            let new_element = existing_element.clone();
            (
                new_element,
                ElementStatus::JustCloned,
                Some(existing_element.ws_node()),
            )
        } else {
            let element = Element::new_ns(self.element_tag);
            (
                element,
                ElementStatus::JustCreated,
                self.end_flag_node.as_ref(),
            )
        };
        let render = ElementRender::new(&self.comp, state, &mut new_element, status);
        (self.fn_render)(item, render);
        new_element.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_element_at(index, new_element);
    }

    fn remove(&mut self, index: usize) {
        let element = self.nodes.remove_element_at(index);
        element.remove_from(&self.parent);
    }

    fn re_render(&mut self, state: &C, index: usize, item: &I) {
        let element = self.nodes.get_element_mut(index);
        let render = ElementRender::new(&self.comp, state, element, ElementStatus::Existing);
        (self.fn_render)(item, render);
    }

    fn move_item(&mut self, old_index: usize, new_index: usize) {
        let element = self.nodes.remove_element_at(old_index);
        let next_sibling = self.nodes.get_element_mut(new_index).ws_node();
        element.insert_before_a_sibling(&self.parent, Some(next_sibling));
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

        let next_sibling = {
            if high_index < self.nodes.count() {
                Some(self.nodes.get_element_mut(high_index).ws_node())
            } else {
                self.end_flag_node.as_ref()
            }
        };
        low_element.insert_before_a_sibling(&self.parent, next_sibling);
        self.nodes.insert_element_at(high_index, low_element);
    }
}
