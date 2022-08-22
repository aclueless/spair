use std::{
    cell::{Cell, RefMut},
    rc::Rc,
};

use super::{QrValueAttribute, QrValueAttributeMap};
use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, WsElement},
    queue_render::{MapValue, QueueRender, Value, ValueContent},
    render::{base::ElementRenderMut, html::HtmlElementRender},
};

impl<'a, C: Component> HtmlElementRender<'a, C> {
    fn qav<T: 'static, Q: 'static + QueueRender<T>>(
        &self,
        value: &Value<T>,
        new: impl FnOnce(Comp<C>, WsElement, Rc<Cell<bool>>) -> Q,
        init: impl FnOnce(&Q, &RefMut<ValueContent<T>>),
    ) {
        let element_render = self.element_render();
        if element_render.status() == ElementStatus::Existing {
            return;
        }
        let element = element_render.element().ws_element().clone();
        let unmounted = element_render.element().unmounted();
        let q = new(self.comp(), element, unmounted);

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                init(&q, &this);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    fn qav_map<T: 'static, U, Q, M: 'static + QueueRender<T>>(
        &self,
        value: MapValue<C, T, U>,
        new: impl FnOnce(Comp<C>, WsElement, Rc<Cell<bool>>) -> Q,
        new_map: impl FnOnce(Q, Box<dyn Fn(&C, &T) -> U>) -> M,
        init: impl FnOnce(&Q, U),
    ) {
        let element_render = self.element_render();
        if element_render.status() == ElementStatus::Existing {
            return;
        }
        let element = element_render.element().ws_element().clone();
        let unmounted = element_render.element().unmounted();
        let q = new(self.comp(), element, unmounted);

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

    pub fn queue_attribute_value_string(&self, value: &Value<String>) {
        self.qav(value, QrValueAttribute::new, |q, value| {
            q.update_str_value(Some(value.value()));
        });
    }

    pub fn queue_attribute_value_optional_string(&self, value: &Value<Option<String>>) {
        self.qav(value, QrValueAttribute::new, |q, value| {
            q.update_str_value(value.value().as_deref());
        });
    }

    pub fn queue_attribute_value_string_map<T: 'static>(&self, value: MapValue<C, T, String>) {
        self.qav_map(
            value,
            QrValueAttribute::new,
            QrValueAttributeMap::new,
            |q, value| {
                q.update_str_value(Some(value.as_str()));
            },
        );
    }
    pub fn queue_attribute_value_optional_string_map<T: 'static>(
        &self,
        value: MapValue<C, T, Option<String>>,
    ) {
        self.qav_map(
            value,
            QrValueAttribute::new,
            QrValueAttributeMap::new,
            |q, value| {
                q.update_str_value(value.as_deref());
            },
        );
    }

    pub fn queue_attribute_selected_index_usize(&self, value: &Value<usize>) {
        self.qav(value, QrValueAttribute::new, |q, value| {
            q.update_usize_index(Some(*value.value()));
        });
    }
    pub fn queue_attribute_selected_index_optional_usize(&self, value: &Value<Option<usize>>) {
        self.qav(value, QrValueAttribute::new, |q, value| {
            q.update_usize_index(*value.value());
        });
    }

    pub fn queue_attribute_selected_index_usize_map<T: 'static>(
        &self,
        value: MapValue<C, T, usize>,
    ) {
        self.qav_map(
            value,
            QrValueAttribute::new,
            QrValueAttributeMap::new,
            |q, value| {
                q.update_usize_index(Some(value));
            },
        );
    }
    pub fn queue_attribute_selected_index_optional_usize_map<T: 'static>(
        &self,
        value: MapValue<C, T, Option<usize>>,
    ) {
        self.qav_map(
            value,
            QrValueAttribute::new,
            QrValueAttributeMap::new,
            |q, value| {
                q.update_usize_index(value);
            },
        );
    }
}
