use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{ELementTag, ElementStatus, Nodes, WsElement},
    queue_render::vec::ListDiff,
    render::base::ElementRender,
};

pub struct QrListRepresentative {
    end_flag_node: Option<web_sys::Node>,
    unmounted: Rc<Cell<bool>>,
}

impl QrListRepresentative {
    pub fn end_flag_node(&self) -> Option<&web_sys::Node> {
        self.end_flag_node.as_ref()
    }

    pub fn mark_as_unmounted(&self) {
        self.unmounted.set(true);
    }
}

pub struct QrList<C: Component, I> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: Nodes,
    end_flag_node: Option<web_sys::Node>,
    element_tag: ELementTag,
    use_template: bool,
    fn_render: Box<dyn Fn(&I, ElementRender<C>)>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, I> QrList<C, I> {
    pub fn new(
        element_tag: ELementTag,
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
        QrListRepresentative {
            end_flag_node: self.end_flag_node.clone(),
            unmounted: self.unmounted.clone(),
        }
    }
    pub fn render(&mut self, diffs: &Vec<ListDiff<I>>) {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrList::render::rc_comp.try_borrow().");
        let state = comp.state();
        for d in diffs.iter() {
            self.render_change(state, d);
        }
    }

    fn render_change(&mut self, state: &C, diff: &ListDiff<I>) {
        match diff {
            ListDiff::Push(item) => self.push(state, item),
            ListDiff::Pop => self.pop(),
            ListDiff::Insert { index, value } => self.insert(state, *index, value),
            ListDiff::RemoveAtIndex(index) => self.remove(*index),
        }
    }

    fn push(&mut self, state: &C, item: &I) {
        let index = self.nodes.count();
        let (namespace, tag) = self.element_tag.namespace_and_tag();
        self.nodes
            .create_new_element_ns(namespace, tag, &self.parent, self.end_flag_node.as_ref());
        let element = self.nodes.get_element_mut(index);
        let render = ElementRender::new(&self.comp, state, element, ElementStatus::JustCreated);
        (self.fn_render)(item, render);
    }

    fn pop(&mut self) {}

    fn insert(&mut self, state: &C, index: usize, item: &I) {
        //
    }

    fn remove(&mut self, index: usize) {
        //
    }
}
