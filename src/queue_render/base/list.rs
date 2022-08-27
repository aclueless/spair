use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{ELementTag, ElementStatus, Nodes},
    queue_render::{
        dom::QrListRepresentative,
        vec::{Diff, ListRender},
    },
    render::base::ElementRender,
};

pub struct QrListRender<C: Component, I> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: Nodes,
    end_flag_node: Option<web_sys::Node>,
    element_tag: ELementTag,
    use_template: bool,
    fn_render: Box<dyn Fn(&I, ElementRender<C>)>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, I> ListRender<I> for QrListRender<C, I> {
    fn render(&mut self, items: &[I], diffs: &[Diff<I>]) {
        self.render_list(items, diffs);
    }

    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component, I> QrListRender<C, I> {
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
        QrListRepresentative::new(self.end_flag_node.clone(), self.unmounted.clone())
    }

    pub fn render_list(&mut self, items: &[I], diffs: &[Diff<I>]) {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrListRender::render::rc_comp.try_borrow().");
        let state = comp.state();
        for d in diffs.iter() {
            self.render_change(state, items, d);
        }
    }

    fn render_change(&mut self, state: &C, items: &[I], diff: &Diff<I>) {
        match diff {
            Diff::New => self.all_new(state, items),
            Diff::Push(item) => self.push(state, item),
            Diff::Pop => self.pop(),
            Diff::Insert { index, value } => self.insert(state, *index, value),
            Diff::RemoveAtIndex(index) => self.remove(*index),
        }
    }

    fn all_new(&mut self, state: &C, items: &[I]) {
        // clear
        // render
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
