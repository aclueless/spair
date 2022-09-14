use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::value::QueueRender,
};

pub trait AttributeRender {
    fn render(&self, name: &str, ws: &WsElement);
}

impl AttributeRender for i32 {
    fn render(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeRender for u32 {
    fn render(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeRender for f64 {
    fn render(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeRender for bool {
    fn render(&self, name: &str, ws: &WsElement) {
        ws.set_bool_attribute(name, *self);
    }
}

impl AttributeRender for String {
    fn render(&self, name: &str, ws: &WsElement) {
        ws.set_str_attribute(name, self);
    }
}

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
}

impl<T: AttributeRender> QueueRender<T> for QrNormalAttribute {
    fn render(&mut self, t: &T) {
        t.render(self.attribute_name, &self.ws_element);
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

impl<C: Component, T, U: AttributeRender> QueueRender<T> for QrNormalAttributeMap<C, T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        u.render(self.qra.attribute_name, &self.qra.ws_element)
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

pub struct QrProperty<T> {
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    fn_update: Box<dyn Fn(&WsElement, &T)>,
}

impl<T> QrProperty<T> {
    pub fn new(
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        fn_update: Box<dyn Fn(&WsElement, &T)>,
    ) -> Self {
        Self {
            unmounted: element_unmounted,
            ws_element,
            fn_update,
        }
    }
}

impl<T> QueueRender<T> for QrProperty<T> {
    fn render(&mut self, t: &T) {
        (self.fn_update)(&self.ws_element, t);
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrPropertyMap<C, T, U>
where
    C: Component,
{
    qr_property: QrProperty<U>,
    comp: Comp<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrPropertyMap<C, T, U> {
    pub fn new(
        qr_property: QrProperty<U>,
        comp: Comp<C>,
        fn_map: Box<dyn Fn(&C, &T) -> U + 'static>,
    ) -> Self {
        Self {
            qr_property,
            comp,
            fn_map,
        }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrPropertyMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T, U> QueueRender<T> for QrPropertyMap<C, T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        (self.qr_property.fn_update)(&self.qr_property.ws_element, &u);
    }
    fn unmounted(&self) -> bool {
        self.qr_property.unmounted.get()
    }
}

pub struct QrClass {
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    last_class: Option<String>,
}

impl QrClass {
    pub fn new(ws_element: WsElement, element_unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            unmounted: element_unmounted,
            ws_element,
            last_class: None,
        }
    }

    pub fn update_str(&mut self, value: Option<&str>) {
        let last = self.last_class.as_deref();
        if value.eq(&last) {
            return;
        }
        self.ws_element.remove_class_optional(last);
        self.ws_element.add_class_optional(value);

        self.last_class = value.map(ToString::to_string);
    }

    pub fn update_string(&mut self, value: Option<String>) {
        if value.eq(&self.last_class) {
            return;
        }
        self.ws_element
            .remove_class_optional(self.last_class.as_deref());
        self.ws_element.add_class_optional(value.as_deref());

        self.last_class = value;
    }
}

impl QueueRender<String> for QrClass {
    fn render(&mut self, t: &String) {
        self.update_str(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<'a> QueueRender<&'a str> for QrClass {
    fn render(&mut self, t: &&str) {
        self.update_str(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl QueueRender<Option<String>> for QrClass {
    fn render(&mut self, t: &Option<String>) {
        self.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrClassMap<C, T, U>
where
    C: Component,
{
    qr: QrClass,
    comp: Comp<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrClassMap<C, T, U> {
    pub fn new(qr: QrClass, comp: Comp<C>, fn_map: Box<dyn Fn(&C, &T) -> U + 'static>) -> Self {
        Self { qr, comp, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrClassMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMap<C, T, &'static str> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_str(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMap<C, T, String> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_string(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMap<C, T, Option<String>> {
    fn render(&mut self, t: &T) {
        let t = self.map(t);
        self.qr.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}
