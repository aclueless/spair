use std::{cell::Cell, rc::Rc};

pub struct QrGroupRepresentative {
    end_flag_node: web_sys::Node,
    unmounted: Rc<Cell<bool>>,
}

impl QrGroupRepresentative {
    pub fn new(end_flag_node: web_sys::Node, unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            end_flag_node,
            unmounted,
        }
    }

    pub fn end_flag_node(&self) -> &web_sys::Node {
        &self.end_flag_node
    }

    pub fn mark_as_unmounted(&self) {
        self.unmounted.set(true);
    }
}
