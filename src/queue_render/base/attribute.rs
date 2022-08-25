use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::QueueRender,
};

pub struct QrNormalAttribute {
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
}

impl QrNormalAttribute {
    pub fn new(
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        attribute_name: &'static str,
    ) -> Self {
        Self {
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
        }
    }

    pub fn update(&self, value: &str) {
        self.ws_element
            .set_str_attribute(self.attribute_name, value);
    }
}

impl<T: ToString> QueueRender<T> for QrNormalAttribute {
    fn render(&mut self, t: &T) {
        self.update(&t.to_string());
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrNormalAttributeMap<C, T, U>
where
    C: Component,
{
    qra: QrNormalAttribute,
    comp: Comp<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrNormalAttributeMap<C, T, U> {
    pub fn new(
        qra: QrNormalAttribute,
        comp: Comp<C>,
        fn_map: Box<dyn Fn(&C, &T) -> U + 'static>,
    ) -> Self {
        Self { qra, comp, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrNormalAttributeMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T, U: ToString> QueueRender<T> for QrNormalAttributeMap<C, T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qra.update(&u.to_string());
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

pub struct QrBoolAttribute {
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
}

impl QrBoolAttribute {
    pub fn new(
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        attribute_name: &'static str,
    ) -> Self {
        Self {
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
        }
    }

    pub fn update(&self, value: bool) {
        self.ws_element
            .set_bool_attribute(self.attribute_name, value);
    }
}

impl QueueRender<bool> for QrBoolAttribute {
    fn render(&mut self, t: &bool) {
        self.update(*t);
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrBoolAttributeMap<C, T>
where
    C: Component,
{
    qra: QrBoolAttribute,
    comp: Comp<C>,
    fn_map: Box<dyn Fn(&C, &T) -> bool>,
}

impl<C: Component, T> QrBoolAttributeMap<C, T> {
    pub fn new(qra: QrBoolAttribute, comp: Comp<C>, fn_map: Box<dyn Fn(&C, &T) -> bool>) -> Self {
        Self { qra, comp, fn_map }
    }

    fn map(&self, value: &T) -> bool {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrBoolAttributeMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T> QueueRender<T> for QrBoolAttributeMap<C, T> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qra.update(u);
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

