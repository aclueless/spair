use std::{
    cell::{Cell, RefMut},
    rc::Rc,
};

use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, WsElement},
    queue_render::{MapValue, QueueRender, Value, ValueContent},
    render::base::ElementRender,
};

use super::{QrBoolAttribute, QrBoolAttributeMap, QrNormalAttribute, QrNormalAttributeMap};

impl<'a, C: Component> ElementRender<'a, C> {
    fn qa<T: 'static + ToString, Q: 'static + QueueRender<T>>(
        &mut self,
        name: &'static str,
        value: &Value<T>,
        new: impl FnOnce(Comp<C>, WsElement, Rc<Cell<bool>>, &'static str) -> Q,
        init: impl FnOnce(&Q, &RefMut<ValueContent<T>>),
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let q = new(self.comp(), element, unmounted, name);

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                init(&q, &this);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    fn qa_map<T: 'static, U: 'static, Q, M: 'static + QueueRender<T>>(
        &mut self,
        name: &'static str,
        value: MapValue<C, T, U>,
        new: impl FnOnce(Comp<C>, WsElement, Rc<Cell<bool>>, &'static str) -> Q,
        new_map: impl FnOnce(Q, Box<dyn Fn(&C, &T) -> U>) -> M,
        init: impl FnOnce(&Q, U),
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }

        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let q = new(self.comp(), element, unmounted, name);

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                init(&q, u);
                let q = new_map(q, fn_map);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn queue_bool_attribute(&mut self, name: &'static str, value: &Value<bool>) {
        self.qa(name, value, QrBoolAttribute::new, |qra, value| {
            qra.update(*value.value());
        })
    }

    pub fn queue_string_attribute(&mut self, name: &'static str, value: &Value<String>) {
        self.qa(name, value, QrNormalAttribute::new, |qra, value| {
            qra.update(value.value());
        });
    }

    pub fn queue_attribute<T: 'static + ToString>(&mut self, name: &'static str, value: &Value<T>) {
        self.qa(name, value, QrNormalAttribute::new, |qra, value| {
            qra.update(&value.value().to_string());
        });
    }

    pub fn queue_bool_attribute_map<T: 'static>(
        &mut self,
        name: &'static str,
        value: MapValue<C, T, bool>,
    ) {
        self.qa_map(
            name,
            value,
            QrBoolAttribute::new,
            QrBoolAttributeMap::new,
            |q, u| {
                q.update(u);
            },
        );
    }

    pub fn queue_attribute_map<T: 'static, U: 'static + ToString>(
        &mut self,
        name: &'static str,
        value: MapValue<C, T, U>,
    ) {
        self.qa_map(
            name,
            value,
            QrNormalAttribute::new,
            QrNormalAttributeMap::new,
            |q, u| {
                q.update(&u.to_string());
            },
        );
    }
}
