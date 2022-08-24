use std::{
    cell::{Cell, RefMut},
    rc::Rc,
};

use super::{QrProperty, QrPropertyMap, QrSelectedValueIndex, QrSelectedValueIndexMap};
use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, WsElement},
    queue_render::{MapValue, QueueRender, Value, ValueContent},
    render::{
        base::{ElementRender, ElementRenderMut},
        html::HtmlElementRender,
    },
};

impl<'a, C: Component> ElementRender<'a, C> {
    fn qr_property<T: 'static>(
        &self,
        value: &Value<T>,
        fn_update: impl Fn(&WsElement, &T) + 'static,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let q = QrProperty::new(element, unmounted, Box::new(fn_update));

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                q.update(this.value());
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    fn qrm_property<T: 'static, U: 'static>(
        &self,
        value: MapValue<C, T, U>,
        fn_update: impl Fn(&WsElement, &U) + 'static,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let q = QrProperty::new(element, unmounted, Box::new(fn_update));

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                q.update(&u);
                this.add_render(Box::new(QrPropertyMap::new(q, self.comp(), fn_map)));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn qr_checked(&self, value: &Value<bool>) {
        self.qr_property(value, WsElement::checked_ref)
    }

    pub fn qrm_checked<T: 'static>(&self, value: MapValue<C, T, bool>) {
        self.qrm_property(value, WsElement::checked_ref)
    }
}

// These methods don't have to be implemented on HtmlElementRender. Because
// they are for queue-render. But their equivalent methods (for incremental
// render) need to be on HtmlElementRender, so these methods need to be on
// HtmlElementRender, too.
impl<'a, C: Component> HtmlElementRender<'a, C> {
    fn qr_value<T: 'static, Q: 'static + QueueRender<T>>(
        &self,
        value: &Value<T>,
        new: impl FnOnce(WsElement, Rc<Cell<bool>>) -> Q,
        init: impl FnOnce(&Q, &RefMut<ValueContent<T>>),
    ) {
        let element_render = self.element_render();
        if element_render.status() == ElementStatus::Existing {
            return;
        }
        let element = element_render.element().ws_element().clone();
        let unmounted = element_render.element().unmounted();
        let q = new(element, unmounted);

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                init(&q, &this);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    fn qr_value_map<T: 'static, U, Q, M: 'static + QueueRender<T>>(
        &self,
        value: MapValue<C, T, U>,
        new: impl FnOnce(WsElement, Rc<Cell<bool>>) -> Q,
        new_map: impl FnOnce(Q, Comp<C>, Box<dyn Fn(&C, &T) -> U>) -> M,
        init: impl FnOnce(&Q, U),
    ) {
        let element_render = self.element_render();
        if element_render.status() == ElementStatus::Existing {
            return;
        }
        let element = element_render.element().ws_element().clone();
        let unmounted = element_render.element().unmounted();
        let q = new(element, unmounted);

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                init(&q, u);
                let q = new_map(q, self.comp(), fn_map);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn qr_selected_value_string(&self, value: &Value<String>) {
        self.qr_value(value, QrSelectedValueIndex::new, |q, value| {
            q.update_str_value(Some(value.value()));
        });
    }

    pub fn qr_selected_value_optional_string(&self, value: &Value<Option<String>>) {
        self.qr_value(value, QrSelectedValueIndex::new, |q, value| {
            q.update_str_value(value.value().as_deref());
        });
    }

    pub fn qrm_selected_value_string<T: 'static>(&self, value: MapValue<C, T, String>) {
        self.qr_value_map(
            value,
            QrSelectedValueIndex::new,
            QrSelectedValueIndexMap::new,
            |q, value| {
                q.update_str_value(Some(value.as_str()));
            },
        );
    }
    pub fn qrm_selected_value_optional_string<T: 'static>(
        &self,
        value: MapValue<C, T, Option<String>>,
    ) {
        self.qr_value_map(
            value,
            QrSelectedValueIndex::new,
            QrSelectedValueIndexMap::new,
            |q, value| {
                q.update_str_value(value.as_deref());
            },
        );
    }

    pub fn qr_selected_index_usize(&self, value: &Value<usize>) {
        self.qr_value(value, QrSelectedValueIndex::new, |q, value| {
            q.update_usize_index(Some(*value.value()));
        });
    }
    pub fn qr_selected_index_optional_usize(&self, value: &Value<Option<usize>>) {
        self.qr_value(value, QrSelectedValueIndex::new, |q, value| {
            q.update_usize_index(*value.value());
        });
    }

    pub fn qrm_selected_index_usize<T: 'static>(&self, value: MapValue<C, T, usize>) {
        self.qr_value_map(
            value,
            QrSelectedValueIndex::new,
            QrSelectedValueIndexMap::new,
            |q, value| {
                q.update_usize_index(Some(value));
            },
        );
    }
    pub fn qrm_selected_index_optional_usize<T: 'static>(
        &self,
        value: MapValue<C, T, Option<usize>>,
    ) {
        self.qr_value_map(
            value,
            QrSelectedValueIndex::new,
            QrSelectedValueIndexMap::new,
            |q, value| {
                q.update_usize_index(value);
            },
        );
    }
}
