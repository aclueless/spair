#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod application;
mod component;
mod dom;
mod events;
mod fetch;
mod renderable;
mod routing;
mod utils;

pub use component::{
    update_component, Checklist, ChildComp, Comp, Component, ShouldRender, WithParentComp,
};
pub use dom::attribute_types::*;
#[cfg(feature = "keyed-list")]
pub use dom::KeyedListItem;
pub use dom::{ElementUpdater as Element, ListElementCreation, Nodes, RawWrapper, StaticNodes};
// TODO selectively export event traits only?
pub use events::*;
pub use fetch::{FetchError, FetchStatus, Request};
pub use renderable::*;
pub use routing::Routes;
pub use utils::{document, into_input, into_select, window};

// pub use web_sys::Location;
// pub use web_sys::{Event, FocusEvent, InputEvent, KeyboardEvent, MouseEvent, UiEvent, WheelEvent};
pub use web_sys;

pub use wasm_bindgen::JsValue;
pub use wasm_bindgen_futures::JsFuture;

pub mod prelude {
    pub use crate::application::Application;
    pub use crate::component::Component;
    pub use crate::dom::{AttributeSetter, DomBuilder};
    pub use crate::fetch::{FetchOptionsSetter, IntoFetchArgs, RawDataMode};
    pub use crate::routing::Routes;
    pub use wasm_bindgen;
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::{JsCast, UnwrapThrowExt};
}

#[must_use = "This value must be returned to the framework. Otherwise, the command will be lost"]
pub struct Command<C>(Box<dyn component::Command<C>>);
#[must_use = "This value must be returned to the framework. Otherwise, the command will be lost"]
pub struct OptionCommand<C>(Option<Box<dyn component::Command<C>>>);

impl<C> From<Command<C>> for OptionCommand<C> {
    fn from(cmd: Command<C>) -> Self {
        OptionCommand(Some(cmd.0))
    }
}

impl<C> From<Command<C>> for Checklist<C>
where
    C: 'static + Component,
{
    fn from(cmd: Command<C>) -> Self {
        let mut checklist = C::default_checklist();
        checklist.add_command(cmd);
        checklist
    }
}

impl<C> From<OptionCommand<C>> for Checklist<C>
where
    C: 'static + Component,
{
    fn from(cmd: OptionCommand<C>) -> Self {
        let mut checklist = C::default_checklist();
        checklist.add_option_command(cmd);
        checklist
    }
}

impl<C> From<Option<Command<C>>> for OptionCommand<C> {
    fn from(cmd: Option<Command<C>>) -> Self {
        OptionCommand(cmd.map(|cmd| cmd.0))
    }
}

pub struct WsRef<T>(std::cell::RefCell<Option<T>>);

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

    pub fn set<C: component::Component>(&self, element: &crate::Element<C>) {
        use wasm_bindgen::JsCast;
        *self.0.borrow_mut() = Some(element.ws_element().unchecked_into());
    }

    pub fn execute(&self, f: impl FnOnce(&T)) {
        if let Some(t) = self.get().as_ref() {
            f(t);
        }
    }
}
