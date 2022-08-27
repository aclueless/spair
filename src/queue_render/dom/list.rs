use std::{cell::Cell, rc::Rc};

pub struct QrListRepresentative {
    end_flag_node: Option<web_sys::Node>,
    unmounted: Rc<Cell<bool>>,
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

    pub fn mark_as_unmounted(&self) {
        self.unmounted.set(true);
    }
}
