use std::{cell::Cell, rc::Rc};

use crate::{component::{Component, Comp, queue_render::QueueRender}, dom::WsElement};

pub struct QrAttribute(Box<dyn QrAttributeNode>);

impl QrAttribute {
    pub fn new(v: Box<dyn QrAttributeNode>) -> Self {
        Self(v)
    }
}

pub trait QrAttributeNode {}

pub struct QrNormalAttribute<C: Component>(Rc<QrNormalAttributeInner<C>>);
pub struct QrNormalAttributeMap<C, T, U>
where
    C: Component,
{
    text_node: QrNormalAttribute<C>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

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
    pub fn new(comp: Comp<C>, ws_element: WsElement, element_unmounted: Rc<Cell<bool>>, attribute_name: &'static str,
    ) -> Self {
        Self(Rc::new(QrNormalAttributeInner {
            comp,
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
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
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
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
    pub fn new(comp: Comp<C>, ws_element: WsElement, element_unmounted: Rc<Cell<bool>>, attribute_name: &'static str,
    ) -> Self {
        Self(Rc::new(QrBoolAttributeInner {
            comp,
            unmounted: element_unmounted,
            ws_element,
            attribute_name,
        }))
    }

    pub fn update(&self, value: bool) {
        self.0.ws_element.set_bool_attribute(self.0.attribute_name, value);
    }
}

impl<C: Component> QrAttributeNode for QrBoolAttribute<C> {}

impl<C: Component> QueueRender<bool> for QrBoolAttribute<C> {
    fn render(&self, t: &bool) {
        self.update(*t);
    }
    fn unmounted(&self) -> bool {
        self.0.unmounted.get()
    }
}

// QrAttributeClass: to remember last class and remove it before setting new class
// QrProperty: for value, id, checked, enabled, disabled?

