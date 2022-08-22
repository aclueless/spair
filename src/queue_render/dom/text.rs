use crate::{
    component::{Comp, Component},
    queue_render::QueueRender,
};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::dom::ParentAndChild;

pub trait QrText: ParentAndChild {
    fn clone_ws_node(&self) -> web_sys::Node;
    fn mark_as_unmounted(&self);
}

pub struct QrTextNode<C: Component>(Rc<TextNodeInner<C>>);

impl<C: Component> Clone for QrTextNode<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct TextNodeInner<C: Component> {
    comp: Comp<C>,
    unmounted: Cell<bool>,
    ws_node: web_sys::Node,
}

impl<C: Component> QrTextNode<C> {
    pub fn new(comp: Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            unmounted: Cell::new(false),
            ws_node: crate::utils::document().create_text_node("").into(),
        }))
    }

    pub fn with_cloned_node(ws_node: web_sys::Node, comp: Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            unmounted: Cell::new(false),
            ws_node,
        }))
    }

    pub fn update_text(&self, text: &str) {
        self.0.ws_node.set_text_content(Some(text)); //.expect_throw();
    }
}

impl<C: Component> ParentAndChild for QrTextNode<C> {
    fn ws_node(&self) -> &web_sys::Node {
        &self.0.ws_node
    }
}

impl<C: Component> QrText for QrTextNode<C> {
    fn clone_ws_node(&self) -> web_sys::Node {
        self.0
            .ws_node
            .clone_node_with_deep(false)
            .expect_throw("dom::queue_render::text::QrText for TextNode::clone_ws_node")
    }

    fn mark_as_unmounted(&self) {
        self.0.unmounted.set(true);
    }
}

impl<C: Component, T: ToString> QueueRender<T> for QrTextNode<C> {
    fn render(&self, t: &T) {
        self.update_text(&t.to_string());
    }
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
    }
}

pub struct QrTextNodeMap<C, T, U>
where
    C: Component,
{
    text_node: QrTextNode<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C, T, U> QrTextNodeMap<C, T, U>
where
    C: Component,
    T: ToString,
    U: 'static,
{
    pub fn new(text_node: QrTextNode<C>, fn_map: impl Fn(&C, &T) -> U + 'static) -> Self {
        Self {
            text_node,
            fn_map: Box::new(fn_map),
        }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.text_node.0.comp.upgrade();
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

impl<C, T, U> QueueRender<T> for QrTextNodeMap<C, T, U>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.update_text(&u.to_string());
    }

    fn unmounted(&self) -> bool {
        self.text_node.0.unmounted.get()
    }
}
