use std::{cell::Cell, rc::Rc};

pub struct QrGroupRepresentative {
    end_flag_node: web_sys::Node,
    unmounted: Rc<Cell<bool>>,
}

impl Drop for QrGroupRepresentative {
    fn drop(&mut self) {
        self.unmounted.set(true);
    }
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
}
