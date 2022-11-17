use std::{cell::Cell, rc::Rc};

pub struct QrListRepresentative {
    end_flag_node: Option<web_sys::Node>,
    unmounted: Rc<Cell<bool>>,
}

impl Drop for QrListRepresentative {
    fn drop(&mut self) {
        self.unmounted.set(true);
    }
}

impl QrListRepresentative {
    pub fn new(end_flag_node: Option<web_sys::Node>, unmounted: Rc<Cell<bool>>) -> Self {
        Self {
            end_flag_node,
            unmounted,
        }
    }

    pub fn end_flag_node(&self) -> Option<&web_sys::Node> {
        self.end_flag_node.as_ref()
    }
}
