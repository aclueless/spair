use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::QueueRender,
};

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

    pub fn update(&self, value: &T) {
        (self.fn_update)(&self.ws_element, value);
    }
}

impl<T> QueueRender<T> for QrProperty<T> {
    fn render(&self, t: &T) {
        self.update(t);
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
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.qr_property.update(&u);
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

    pub fn update_str(&self, value: Option<&str>) {
        todo!("Must be mutable or last class as RefCell")
    }

    pub fn update_string(&self, value: Option<String>) {
        todo!("Must be mutable or last class as RefCell")
    }
}

impl QueueRender<String> for QrClass {
    fn render(&self, t: &String) {
        self.update_str(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl QueueRender<Option<String>> for QrClass {
    fn render(&self, t: &Option<String>) {
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
    pub fn new(
        qr: QrClass,
        comp: Comp<C>,
        fn_map: Box<dyn Fn(&C, &T) -> U + 'static>,
    ) -> Self {
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

impl<C: Component, T> QueueRender<T> for QrClassMap<C, T, String> {
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.qr.update_string(Some(u));
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrClassMap<C, T, Option<String>> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qr.update_str(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

