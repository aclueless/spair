use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::QueueRender,
};

pub struct QrNormalAttribute<C: Component>(Rc<QrNormalAttributeInner<C>>);

impl<C: Component> Clone for QrNormalAttribute<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// For attributes that can be updated with WsElement::set_str_attribute
struct QrNormalAttributeInner<C: Component> {
    comp: Comp<C>,
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
}

impl<C: Component> QrNormalAttribute<C> {
    pub fn new(
        comp: Comp<C>,
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        attribute_name: &'static str,
    ) -> Self {
        Self(Rc::new(QrNormalAttributeInner {
            comp,
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
        }))
    }

    pub fn update(&self, value: &str) {
        self.0
            .ws_element
            .set_str_attribute(self.0.attribute_name, value);
    }
}

impl<C: Component, T: ToString> QueueRender<T> for QrNormalAttribute<C> {
    fn render(&self, t: &T) {
        self.update(&t.to_string());
    }
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
    }
}

pub struct QrNormalAttributeMap<C, T, U>
where
    C: Component,
{
    qra: QrNormalAttribute<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrNormalAttributeMap<C, T, U> {
    pub fn new(qra: QrNormalAttribute<C>, fn_map: Box<dyn Fn(&C, &T) -> U + 'static>) -> Self {
        Self { qra, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.qra.0.comp.upgrade();
        let comp = rc_comp
            .try_borrow_mut()
            .expect_throw("QrNormalAttributeMap::map::rc_comp.try_borrow_mut().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }

    pub fn update(&self, value: &str) {
        self.qra.update(value);
    }
}

impl<C: Component, T, U: ToString> QueueRender<T> for QrNormalAttributeMap<C, T, U> {
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.update(&u.to_string());
    }
    fn unmounted(&self) -> bool {
        self.qra.0.unmounted.get()
    }
}

pub struct QrBoolAttribute<C: Component>(Rc<QrBoolAttributeInner<C>>);

impl<C: Component> Clone for QrBoolAttribute<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct QrBoolAttributeInner<C: Component> {
    comp: Comp<C>,
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
}

impl<C: Component> QrBoolAttribute<C> {
    pub fn new(
        comp: Comp<C>,
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        attribute_name: &'static str,
    ) -> Self {
        Self(Rc::new(QrBoolAttributeInner {
            comp,
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
        }))
    }

    pub fn update(&self, value: bool) {
        self.0
            .ws_element
            .set_bool_attribute(self.0.attribute_name, value);
    }
}

impl<C: Component> QueueRender<bool> for QrBoolAttribute<C> {
    fn render(&self, t: &bool) {
        self.update(*t);
    }
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
    }
}

pub struct QrBoolAttributeMap<C, T>
where
    C: Component,
{
    qra: QrBoolAttribute<C>,
    fn_map: Box<dyn Fn(&C, &T) -> bool>,
}

impl<C: Component, T> QrBoolAttributeMap<C, T> {
    pub fn new(qra: QrBoolAttribute<C>, fn_map: Box<dyn Fn(&C, &T) -> bool>) -> Self {
        Self { qra, fn_map }
    }

    fn map(&self, value: &T) -> bool {
        let rc_comp = self.qra.0.comp.upgrade();
        let comp = rc_comp
            .try_borrow_mut()
            .expect_throw("QrBoolAttributeMap::map::rc_comp.try_borrow_mut().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }

    pub fn update(&self, value: bool) {
        self.qra.update(value);
    }
}

impl<C: Component, T> QueueRender<T> for QrBoolAttributeMap<C, T> {
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.update(u);
    }
    fn unmounted(&self) -> bool {
        self.qra.0.unmounted.get()
    }
}

// QrAttributeClass: to remember last class and remove it before setting new class
// QrProperty: for value, id, checked, enabled, disabled?
