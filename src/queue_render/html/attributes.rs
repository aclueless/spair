use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::QueueRender,
};

pub struct QrSelectedValueIndex {
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
}

impl QrSelectedValueIndex {
    pub fn new(ws_element: WsElement, element_unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            unmounted: element_unmounted,
            ws_element,
        }
    }

    pub fn update_str_value(&self, value: Option<&str>) {
        match value {
            None => self.ws_element.set_selected_index(-1),
            Some(value) => {
                let _users_must_update_the_selected_value_after_the_list =
                    self.ws_element.set_value(value, true);
            }
        }
    }

    pub fn update_usize_index(&self, index: Option<usize>) {
        match index {
            None => self.ws_element.set_selected_index(-1),
            Some(index) => self.ws_element.set_selected_index(index as i32),
        }
    }
}

impl QueueRender<String> for QrSelectedValueIndex {
    fn render(&self, t: &String) {
        self.update_str_value(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl QueueRender<Option<String>> for QrSelectedValueIndex {
    fn render(&self, t: &Option<String>) {
        self.update_str_value(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl QueueRender<usize> for QrSelectedValueIndex {
    fn render(&self, t: &usize) {
        self.update_usize_index(Some(*t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl QueueRender<Option<usize>> for QrSelectedValueIndex {
    fn render(&self, t: &Option<usize>) {
        self.update_usize_index(*t);
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrSelectedValueIndexMap<C, T, U>
where
    C: Component,
{
    qr: QrSelectedValueIndex,
    comp: Comp<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrSelectedValueIndexMap<C, T, U> {
    pub fn new(
        qr: QrSelectedValueIndex,
        comp: Comp<C>,
        fn_map: Box<dyn Fn(&C, &T) -> U + 'static>,
    ) -> Self {
        Self { qr, comp, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.comp.upgrade();
        let comp = rc_comp
            .try_borrow()
            .expect_throw("QrSelectedValueIndexMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }
}

impl<C: Component, T> QueueRender<T> for QrSelectedValueIndexMap<C, T, String> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qr.update_str_value(Some(&t));
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrSelectedValueIndexMap<C, T, Option<String>> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qr.update_str_value(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrSelectedValueIndexMap<C, T, usize> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qr.update_usize_index(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrSelectedValueIndexMap<C, T, Option<usize>> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qr.update_usize_index(t);
    }
    fn unmounted(&self) -> bool {
        self.qr.unmounted.get()
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
        let t = self.map(t);
        self.qr_property.update(&t);
    }
    fn unmounted(&self) -> bool {
        self.qr_property.unmounted.get()
    }
}
