use crate::{
    component::{Comp, Component},
    queue_render::{FnMap, FnMapC, val::QueueRender},
};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::dom::AChildNode;

pub struct QrTextNode(Rc<TextNodeInner>);

impl Drop for QrTextNode {
    fn drop(&mut self) {
        self.0.unmounted.set(true);
    }
}

impl Clone for QrTextNode {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct TextNodeInner {
    unmounted: Cell<bool>,
    ws_node: web_sys::Node,
}

impl Default for QrTextNode {
    fn default() -> Self {
        Self(Rc::new(TextNodeInner {
            unmounted: Cell::new(false),
            ws_node: crate::utils::document().create_text_node("").into(),
        }))
    }
}

impl QrTextNode {
    pub fn with_cloned_node(ws_node: web_sys::Node) -> Self {
        Self(Rc::new(TextNodeInner {
            unmounted: Cell::new(false),
            ws_node,
        }))
    }

    pub fn update_text(&self, text: &str) {
        self.0.ws_node.set_text_content(Some(text));
    }

    pub fn clone_ws_node(&self) -> web_sys::Node {
        self.0
            .ws_node
            .clone_node_with_deep(false)
            .expect_throw("dom::queue_render::text::QrTextNode for QrTextNode::clone_ws_node")
    }
}

impl AChildNode for QrTextNode {
    fn ws_node(&self) -> &web_sys::Node {
        &self.0.ws_node
    }
}

impl<T: ToString> QueueRender<T> for QrTextNode {
    fn render(&mut self, t: &T) {
        self.update_text(&t.to_string());
    }
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
    }
}

pub struct QrTextNodeMap<T, U> {
    text_node: QrTextNode,
    fn_map: FnMap<T,U>,
}

impl<T, U> QrTextNodeMap<T, U>
where
    T: ToString,
    U: 'static,
{
    pub fn new(text_node: QrTextNode, fn_map: impl Fn(&T) -> U + 'static) -> Self {
        Self {
            text_node,
            fn_map: Box::new(fn_map),
        }
    }

    pub fn map(&self, value: &T) -> U {
        (self.fn_map)(value)
    }

    pub fn update_text(&self, text: &str) {
        self.text_node.update_text(text);
    }
}

impl<T, U> QueueRender<T> for QrTextNodeMap<T, U>
where
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.update_text(&u.to_string());
    }

    fn unmounted(&self) -> bool {
        self.text_node.0.unmounted.get()
    }
}

pub struct QrTextNodeMapWithState<C, T, U>
where
    C: Component,
{
    text_node: QrTextNode,
    comp: Comp<C>,
    fn_map: FnMapC<C, T, U>,
}

impl<C, T, U> QrTextNodeMapWithState<C, T, U>
where
    C: Component,
    T: ToString,
    U: 'static,
{
    pub fn new(
        text_node: QrTextNode,
        comp: Comp<C>,
        fn_map: impl Fn(&C, &T) -> U + 'static,
    ) -> Self {
        Self {
            text_node,
            comp,
            fn_map: Box::new(fn_map),
        }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrTextNodeMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }

    pub fn map_with_state(&self, state: &C, value: &T) -> U {
        (self.fn_map)(state, value)
    }

    pub fn update_text(&self, text: &str) {
        self.text_node.update_text(text);
    }
}

impl<C, T, U> QueueRender<T> for QrTextNodeMapWithState<C, T, U>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.update_text(&u.to_string());
    }

    fn unmounted(&self) -> bool {
        self.text_node.0.unmounted.get()
    }
}
