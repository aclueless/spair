use std::{cell::Cell, rc::Rc};

use crate::{component::{Component, Comp}, dom::WsElement};

pub struct QrAttribute {
}

pub struct QrNormalAttribute<C: Component, T>(Rc<QrNormalAttributeInner<C, T>>);

impl<C: Component, T> Clone for QrNormalAttribute<C, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct QrNormalAttributeInner<C: Component, T> {
    comp: Comp<C>,
    dropped: Rc<Cell<bool>>,
    ws_element: WsElement,
    attribute_name: &'static str,
    fn_update: fn(&WsElement, &str, &T)
}


/*
ClassAttribute: to remember last class and remove it before setting new class
Attribute:
Property: for value, id, checked, enabled, disabled?
*/


