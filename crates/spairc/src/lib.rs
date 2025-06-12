#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod component;
mod element;
mod events;
mod helper;
mod keyed_list;
mod list;
mod routing;

#[cfg(test)]
mod test_helper;

use std::{cell::RefCell, ops::Deref};

pub use wasm_bindgen;
use wasm_bindgen::JsCast;
pub use web_sys;

pub use crate::helper::WINDOW;
pub use component::{
    Callback, CallbackArg, Comp, Component, Context, start_app, start_app_with_routing,
};
pub use element::{Element, TemplateElement, Text, WsElement, WsNode, WsText};
pub use keyed_list::KeyedList;
pub use list::List;
pub use routing::Route;

pub mod prelude {
    pub use crate::component::{
        Callback, CallbackArg, Comp, Context, RcComp, ShouldRender, SpairSpawnLocal,
        SpairSpawnLocalWithCallback,
    };
    pub use crate::element::RenderOptionWithDefault;
    pub use crate::helper::ElementFromCurrentEventTarget;
    pub use spair_macros::*;
}

pub struct WsRef<T>(RefCell<Option<T>>);
impl<T: wasm_bindgen::JsCast> Default for WsRef<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T: wasm_bindgen::JsCast> WsRef<T> {
    pub fn none() -> Self {
        Self(std::cell::RefCell::new(None))
    }

    pub fn get(&self) -> std::cell::Ref<Option<T>> {
        self.0.borrow()
    }

    pub fn set(&self, element: &WsElement) {
        let e = element.deref().clone().unchecked_into::<T>();
        *self.0.borrow_mut() = Some(e);
    }

    pub fn execute<O>(&self, f: impl FnOnce(&T) -> O) -> Option<O> {
        self.get().as_ref().map(f)
    }
}
