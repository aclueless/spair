use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::{FnMapC, val::QueueRender},
};

pub trait AttributeUpdater {
    fn update(&self, name: &str, ws: &WsElement);
}

impl AttributeUpdater for i32 {
    fn update(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeUpdater for u32 {
    fn update(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeUpdater for f64 {
    fn update(&self, name: &str, ws: &WsElement) {
        ws.set_attribute(name, *self);
    }
}

impl AttributeUpdater for bool {
    fn update(&self, name: &str, ws: &WsElement) {
        ws.set_bool_attribute(name, *self);
    }
}

impl AttributeUpdater for String {
    fn update(&self, name: &str, ws: &WsElement) {
        ws.set_str_attribute(name, self);
    }
}

pub struct QrNormalAttribute {
    element_unmounted: Rc<Cell<bool>>,
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
            element_unmounted,
            ws_element,
            attribute_name,
        }
    }
}

impl<T: AttributeUpdater> QueueRender<T> for QrNormalAttribute {
    fn render(&mut self, t: &T) {
        t.update(self.attribute_name, &self.ws_element);
    }

    fn unmounted(&self) -> bool {
        self.element_unmounted.get()
    }
}

pub struct QrNormalAttributeMap<T, U> {
    qra: QrNormalAttribute,
    fn_map: Box<dyn Fn(&T) -> U>,
}

impl<T, U> QrNormalAttributeMap<T, U> {
    pub fn new(qra: QrNormalAttribute, fn_map: Box<dyn Fn(&T) -> U + 'static>) -> Self {
        Self { qra, fn_map }
    }

    fn map(&self, value: &T) -> U {
        (self.fn_map)(value)
    }
}

impl<T, U: AttributeUpdater> QueueRender<T> for QrNormalAttributeMap<T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        u.update(self.qra.attribute_name, &self.qra.ws_element)
    }
    fn unmounted(&self) -> bool {
        self.qra.element_unmounted.get()
    }
}

pub struct QrNormalAttributeMapWithState<C, T, U>
where
    C: Component,
{
    qra: QrNormalAttribute,
    comp: Comp<C>,
    fn_map: FnMapC<C, T, U>,
}

impl<C: Component, T, U> QrNormalAttributeMapWithState<C, T, U> {
    pub fn new(
        qra: QrNormalAttribute,
        comp: Comp<C>,
        fn_map: FnMapC<C, T, U>,
    ) -> Self {
        Self { qra, comp, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrNormalAttributeMapWithState::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T, U: AttributeUpdater> QueueRender<T>
    for QrNormalAttributeMapWithState<C, T, U>
{
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        u.update(self.qra.attribute_name, &self.qra.ws_element)
    }
    fn unmounted(&self) -> bool {
        self.qra.element_unmounted.get()
    }
}

type FnWsElementUpdater<T> = Box<dyn Fn(&WsElement, &T)>;

pub struct QrProperty<T> {
    element_unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    fn_update: FnWsElementUpdater<T>,
}

impl<T> QrProperty<T> {
    pub fn new(
        ws_element: WsElement,
        element_unmounted: Rc<Cell<bool>>,
        fn_update: FnWsElementUpdater<T>,
    ) -> Self {
        Self {
            element_unmounted,
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
        self.element_unmounted.get()
    }
}

pub struct QrPropertyMap<T, U> {
    qr_property: QrProperty<U>,
    fn_map: Box<dyn Fn(&T) -> U>,
}

impl<T, U> QrPropertyMap<T, U> {
    pub fn new(qr_property: QrProperty<U>, fn_map: Box<dyn Fn(&T) -> U + 'static>) -> Self {
        Self {
            qr_property,
            fn_map,
        }
    }

    fn map(&self, value: &T) -> U {
        (self.fn_map)(value)
    }
}

impl<T, U> QueueRender<T> for QrPropertyMap<T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        (self.qr_property.fn_update)(&self.qr_property.ws_element, &u);
    }
    fn unmounted(&self) -> bool {
        self.qr_property.element_unmounted.get()
    }
}

pub struct QrPropertyMapWithState<C, T, U>
where
    C: Component,
{
    qr_property: QrProperty<U>,
    comp: Comp<C>,
    fn_map: FnMapC<C, T, U>,
}

impl<C: Component, T, U> QrPropertyMapWithState<C, T, U> {
    pub fn new(
        qr_property: QrProperty<U>,
        comp: Comp<C>,
        fn_map: FnMapC<C, T, U>,
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
            .expect_throw("QrPropertyMapWithState::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T, U> QueueRender<T> for QrPropertyMapWithState<C, T, U> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        (self.qr_property.fn_update)(&self.qr_property.ws_element, &u);
    }
    fn unmounted(&self) -> bool {
        self.qr_property.element_unmounted.get()
    }
}

pub struct QrClass {
    element_unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
    last_class: Option<String>,
}

impl QrClass {
    pub fn new(ws_element: WsElement, element_unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            element_unmounted,
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
        self.element_unmounted.get()
    }
}

impl<'a> QueueRender<&'a str> for QrClass {
    fn render(&mut self, t: &&str) {
        self.update_str(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.element_unmounted.get()
    }
}

impl QueueRender<Option<String>> for QrClass {
    fn render(&mut self, t: &Option<String>) {
        self.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.element_unmounted.get()
    }
}

pub struct QrClassMap<T, U> {
    qr: QrClass,
    fn_map: Box<dyn Fn(&T) -> U>,
}

impl<T, U> QrClassMap<T, U> {
    pub fn new(qr: QrClass, fn_map: Box<dyn Fn(&T) -> U + 'static>) -> Self {
        Self { qr, fn_map }
    }

    fn map(&self, value: &T) -> U {
        (self.fn_map)(value)
    }
}

impl<T> QueueRender<T> for QrClassMap<T, &'static str> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_str(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}

impl<T> QueueRender<T> for QrClassMap<T, String> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_string(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}

impl<T> QueueRender<T> for QrClassMap<T, Option<String>> {
    fn render(&mut self, t: &T) {
        let t = self.map(t);
        self.qr.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}

pub struct QrClassMapWithState<C, T, U>
where
    C: Component,
{
    qr: QrClass,
    comp: Comp<C>,
    fn_map: FnMapC<C, T, U>,
}

impl<C: Component, T, U> QrClassMapWithState<C, T, U> {
    pub fn new(qr: QrClass, comp: Comp<C>, fn_map: FnMapC<C, T, U>) -> Self {
        Self { qr, comp, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrClassMapWithState::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMapWithState<C, T, &'static str> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_str(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMapWithState<C, T, String> {
    fn render(&mut self, t: &T) {
        let u = self.map(t);
        self.qr.update_string(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMapWithState<C, T, Option<String>> {
    fn render(&mut self, t: &T) {
        let t = self.map(t);
        self.qr.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qr.element_unmounted.get()
    }
}
