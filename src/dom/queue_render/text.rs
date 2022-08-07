use crate::component::{queue_render::QueueRender, Comp, Component};
use std::{cell::Cell, marker::PhantomData, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::dom::ParentAndChild;

pub trait QrText: ParentAndChild {
    // fn remove_from(&self, parent: &web_sys::Node);
    // fn append_to(&self, parent: &web_sys::Node);
    // fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>);

    // fn ws_node(&self) -> &web_sys::Node;
    fn clone_ws_node(&self) -> web_sys::Node;
}

pub struct QrTextNode<C: Component>(Rc<TextNodeInner<C>>);
impl<C: Component> Clone for QrTextNode<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct TextNodeInner<C: Component> {
    comp: Comp<C>,
    dropped: Cell<bool>,
    ws_node: web_sys::Node,
}

impl<C: Component> QrTextNode<C> {
    pub fn new(comp: Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            dropped: Cell::new(false),
            ws_node: crate::utils::document().create_text_node("").into(),
        }))
    }

    pub fn with_cloned_node(ws_node: web_sys::Node, comp: Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            dropped: Cell::new(false),
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
    // fn remove_from(&self, parent: &web_sys::Node) {
    //     self.0.dropped.set(true);
    //     parent
    //         .remove_child(&self.0.ws_node)
    //         .expect_throw("Unable to remove a child Element from its parent");
    // }

    // fn append_to(&self, parent: &web_sys::Node) {
    //     parent
    //         .append_child(&self.0.ws_node)
    //         .expect_throw("Unable to append a child Text to its expected parent");
    // }

    // fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
    //     parent
    //         .insert_before(&self.0.ws_node, next_sibling)
    //         .expect_throw("Unable to insert a child Text to its expected parent");
    // }

    // fn get_first_element(&self) -> Option<&super::Element> {
    //     None
    // }

    fn clone_ws_node(&self) -> web_sys::Node {
        self.0
            .ws_node
            .clone_node_with_deep(false)
            .expect_throw("dom::queue_render::text::QrText for TextNode::clone_ws_node")
    }
}

impl<C: Component, T: ToString> QueueRender<T> for QrTextNode<C> {
    fn render(&self, t: &T) {
        self.update_text(&t.to_string());
    }
    fn dropped(&self) -> bool {
        self.0.dropped.get()
    }
}

pub struct QrTextNodeMap<C, T, U, F>
where
    C: Component,
    F: Fn(&C, &T) -> U,
{
    text_node: QrTextNode<C>,
    fn_map: F,
    phantom: PhantomData<dyn Fn(C, T) -> U>,
}

impl<C, T, U, F> QrTextNodeMap<C, T, U, F>
where
    C: Component,
    T: ToString,
    F: Fn(&C, &T) -> U,
{
    pub fn new(text_node: QrTextNode<C>, fn_map: F) -> Self {
        Self {
            text_node,
            fn_map,
            phantom: PhantomData,
        }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.text_node.0.comp.upgrade();
        let comp = rc_comp
            .try_borrow_mut()
            .expect_throw("QrTextNodeMap::map::rc_comp.try_borrow_mut().");
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

impl<C, T, U, F> QueueRender<T> for QrTextNodeMap<C, T, U, F>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
    F: 'static + Fn(&C, &T) -> U,
{
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.update_text(&u.to_string());
    }

    fn dropped(&self) -> bool {
        self.text_node.0.dropped.get()
    }
}
