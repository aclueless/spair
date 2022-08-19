use std::{cell::Cell, rc::Rc};

use crate::component::{Component, Comp};

pub struct QrAttribute {
}

pub struct QrAttributeManager<C: Component, T> {
    comp: Comp<C>,
    ws_element: web_sys::Element,
    dropped: Rc<Cell<bool>>,
    update: Box<dyn Fn(web_sys::Element, T)>,
}

/*
ClassAttribute: to remember last class and remove it before setting new class
Attribute:
Property: for value, id, checked, enabled, disabled?
*/
