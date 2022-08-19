use std::{cell::Cell, rc::Rc};

use crate::{component::{Component, Comp, queue_render::QueueRender}, dom::WsElement};

pub struct QrAttribute(Box<dyn QrAttributeNode>);

pub trait QrAttributeNode {}

pub struct QrNormalAttribute<C: Component>(Rc<QrNormalAttributeInner<C>>);

impl<C: Component> Clone for QrNormalAttribute<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// For attributes that can be updated with WsElement::set_str_attribute
struct QrNormalAttributeInner<C: Component> {
    comp: Comp<C>,
    dropped: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
    //fn_update: fn(&WsElement, &str, &T)
}

impl<C: Component> QrNormalAttribute<C> {
    pub fn new(comp: Comp<C>, ws_element: WsElement, element_dropped: Rc<Cell<bool>>, attribute_name: &'static str,
    //fn_update: fn(&WsElement, &str, &T)
    ) -> Self {
        Self(Rc::new(QrNormalAttributeInner {
            comp,
            dropped: element_dropped,
            ws_element,
            attribute_name,
            //fn_update,
        }))
    }

    pub fn update(&self, value: &str) {
        self.0.ws_element.set_str_attribute(self.0.attribute_name, value);
    }
}

impl<C: Component> QrAttributeNode for QrNormalAttribute<C> {}

impl<C: Component, T: ToString> QueueRender<T> for QrNormalAttribute<C> {
    fn render(&self, t: &T) {
        self.update(&t.to_string());
    }
    fn dropped(&self) -> bool {
        self.0.dropped.get()
    }
}

/*
ClassAttribute: to remember last class and remove it before setting new class
Attribute:
Property: for value, id, checked, enabled, disabled?
*/


