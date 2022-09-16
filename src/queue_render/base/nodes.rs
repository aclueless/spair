use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::{AChildNode, ElementTag, GroupedNodes},
    queue_render::{
        base::QrListRender,
        dom::{QrGroupRepresentative, QrNode, QrTextNode},
        value::QueueRender,
    },
    render::{
        base::{ElementUpdater, MatchIfUpdater, NodesUpdater},
        ListElementCreation,
    },
};

impl<'a, C: Component> NodesUpdater<'a, C> {
    // This method is called by incremental-dom, so it will return a new QrTextNode
    // on: New, or on Clone. If the node is an existing active node, it returns None.
    pub fn create_qr_text_node(&mut self) -> Option<QrTextNode> {
        let tn = if self.new_node() {
            let tn = QrTextNode::new();
            tn.insert_before_a_sibling(self.parent(), self.next_sibling());
            self.nodes_mut().add_qr_node(QrNode::Text(tn.clone()));
            Some(tn)
        } else {
            let index = self.index();
            let qr_node = self.nodes_mut().get_qr_node(index);
            match qr_node {
                QrNode::Text(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let tn = QrTextNode::with_cloned_node(wsn);
                        *qr_node = QrNode::Text(tn.clone());
                        Some(tn)
                    }
                    None => None,
                },
                QrNode::List(_) | QrNode::Group(_) => {
                    panic!("spair internal error: Expect a ClonedWsNode or Text");
                }
            }
        };
        self.next_index();
        tn
    }

    pub fn create_qr_list_render<E: ElementTag, I, R>(
        &mut self,
        full_list: bool,
        mode: ListElementCreation,
        tag: E,
        fn_render: R,
    ) -> Option<QrListRender<C, E, I>>
    where
        I: Clone,
        //for<'i, 'r> R: 'static + Fn(&'i I, ElementUpdater<'r, C>),
        R: 'static + Fn(I, ElementUpdater<C>),
    {
        let list = if self.new_node() {
            let end_flag_node = if full_list {
                None
            } else {
                let n: web_sys::Node = crate::utils::document()
                    .create_comment("Mark the end of a qr list")
                    .into();
                n.insert_before_a_sibling(self.parent(), self.next_sibling());
                Some(n)
            };

            let list = QrListRender::new(
                tag,
                self.comp(),
                self.parent().clone(),
                end_flag_node,
                fn_render,
                mode.use_template(),
            );
            self.nodes_mut()
                .add_qr_node(QrNode::List(list.make_representative()));
            Some(list)
        } else {
            let index = self.index();
            let comp = self.comp();
            let parent = self.parent().clone();
            let qr_node = self.nodes_mut().get_qr_node(index);
            match qr_node {
                QrNode::List(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let list = QrListRender::new(
                            tag,
                            comp,
                            parent,
                            Some(wsn),
                            fn_render,
                            mode.use_template(),
                        );
                        *qr_node = QrNode::List(list.make_representative());
                        Some(list)
                    }
                    None => None,
                },
                QrNode::Text(_) | QrNode::Group(_) => {
                    panic!("spair internal error: Expect a ClonedWsNode or List");
                }
            }
        };
        self.next_index();
        list
    }

    pub fn create_qr_match_if<T, R>(&mut self, fn_render: R) -> Option<QrMatchIfUpdater<C, T>>
    where
        for<'t, 'r> R: 'static + Fn(&'t T, MatchIfUpdater<'r, C>),
    {
        let group = if self.new_node() {
            let r = QrMatchIfUpdater {
                comp: self.comp(),
                parent: self.parent().clone(),
                nodes: GroupedNodes::new(),
                fn_render: Box::new(fn_render),
                unmounted: Rc::new(Cell::new(false)),
            };
            r.nodes
                .end_flag_node()
                .insert_before_a_sibling(self.parent(), self.next_sibling());
            self.nodes_mut()
                .add_qr_node(QrNode::Group(r.make_representative()));
            Some(r)
        } else {
            let index = self.index();
            let comp = self.comp();
            let parent = self.parent().clone();
            let qr_node = self.nodes_mut().get_qr_node(index);
            match qr_node {
                QrNode::Group(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let r = QrMatchIfUpdater {
                            comp,
                            parent,
                            nodes: GroupedNodes::with_flag(wsn),
                            fn_render: Box::new(fn_render),
                            unmounted: Rc::new(Cell::new(false)),
                        };
                        *qr_node = QrNode::Group(r.make_representative());
                        Some(r)
                    }
                    None => None,
                },
                QrNode::Text(_) | QrNode::List(_) => {
                    panic!("spair internal error: Expect a ClonedWsNode or Group");
                }
            }
        };
        self.next_index();
        group
    }
}

pub struct QrMatchIfUpdater<C: Component, T> {
    comp: Comp<C>,
    parent: web_sys::Node,
    nodes: GroupedNodes,
    fn_render: Box<dyn Fn(&T, MatchIfUpdater<C>)>,
    unmounted: Rc<Cell<bool>>,
}

impl<C: Component, T> QrMatchIfUpdater<C, T> {
    pub fn make_representative(&self) -> QrGroupRepresentative {
        QrGroupRepresentative::new(self.nodes.end_flag_node().clone(), self.unmounted.clone())
    }
}

impl<C: Component, T> QueueRender<T> for QrMatchIfUpdater<C, T> {
    fn render(&mut self, t: &T) {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrListRender::render::rc_comp.try_borrow().");
        let state = comp.state();
        let mi = MatchIfUpdater::new(&self.comp, state, &self.parent, &mut self.nodes);
        (self.fn_render)(t, mi);
    }

    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}
