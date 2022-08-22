use std::{cell::Cell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    queue_render::QueueRender,
};

pub struct QrValueAttribute<C: Component> {
    comp: Comp<C>,
    unmounted: Rc<Cell<bool>>,
    ws_element: WsElement,
}

impl<C: Component> QrValueAttribute<C> {
    pub fn new(comp: Comp<C>, ws_element: WsElement, element_unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            comp,
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

impl<C: Component> QueueRender<String> for QrValueAttribute<C> {
    fn render(&self, t: &String) {
        self.update_str_value(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component> QueueRender<Option<String>> for QrValueAttribute<C> {
    fn render(&self, t: &Option<String>) {
        self.update_str_value(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component> QueueRender<usize> for QrValueAttribute<C> {
    fn render(&self, t: &usize) {
        self.update_usize_index(Some(*t));
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

impl<C: Component> QueueRender<Option<usize>> for QrValueAttribute<C> {
    fn render(&self, t: &Option<usize>) {
        self.update_usize_index(*t);
    }
    fn unmounted(&self) -> bool {
        self.unmounted.get()
    }
}

pub struct QrValueAttributeMap<C, T, U>
where
    C: Component,
{
    qra: QrValueAttribute<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C: Component, T, U> QrValueAttributeMap<C, T, U> {
    pub fn new(qra: QrValueAttribute<C>, fn_map: Box<dyn Fn(&C, &T) -> U + 'static>) -> Self {
        Self { qra, fn_map }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.qra.comp.upgrade();
        let comp = rc_comp
            .try_borrow() // TODO: change other instance to try_borrow also
            .expect_throw("QrValueAttributeMap::map::rc_comp.try_borrow().");
        let state = comp.state();
        (self.fn_map)(state, value)
    }

    // pub fn update_str_value(&self, value: Option<&str>) {
    //     self.qra.update_str_value(value);
    // }
}

impl<C: Component, T> QueueRender<T> for QrValueAttributeMap<C, T, String> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qra.update_str_value(Some(&t));
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrValueAttributeMap<C, T, Option<String>> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qra.update_str_value(t.as_deref());
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrValueAttributeMap<C, T, usize> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qra.update_usize_index(Some(t));
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}

impl<C: Component, T> QueueRender<T> for QrValueAttributeMap<C, T, Option<usize>> {
    fn render(&self, t: &T) {
        let t = self.map(t);
        self.qra.update_usize_index(t);
    }
    fn unmounted(&self) -> bool {
        self.qra.unmounted.get()
    }
}
