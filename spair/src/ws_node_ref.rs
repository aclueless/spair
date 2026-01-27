use std::cell::RefCell;

use wasm_bindgen::JsCast;

use crate::{WsElement, WsNodeFns};

pub struct WsRef<T>(RefCell<Option<T>>);
impl<T> Default for WsRef<T>
where
    T: JsCast,
{
    fn default() -> Self {
        Self::none()
    }
}

impl<T> WsRef<T>
where
    T: JsCast,
{
    pub fn none() -> Self {
        Self(std::cell::RefCell::new(None))
    }

    pub fn get(&'_ self) -> std::cell::Ref<'_, Option<T>> {
        self.0.borrow()
    }

    pub fn set(&self, element: &WsElement) {
        let e = element.get_ws_node_ref().clone().unchecked_into::<T>();
        *self.0.borrow_mut() = Some(e);
    }

    pub fn execute<O>(&self, f: impl FnOnce(&T) -> O) -> Option<O> {
        self.get().as_ref().map(f)
    }
}
