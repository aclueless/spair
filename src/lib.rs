#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod application;
mod callback;
mod component;
mod dom;
mod events;
mod fetch;
mod future;
mod macros;
mod render;
mod routing;
mod utils;

#[cfg(feature = "queue-render")]
mod queue_render;

pub use application::Application;
pub use component::{
    AsChildComp, Checklist, Child, ChildComp, Comp, Component, ELementTag, ShouldRender,
};
#[cfg(feature = "keyed-list")]
pub use dom::Keyed;
#[cfg(feature = "queue-render")]
pub use queue_render::Value;
#[cfg(feature = "svg")]
pub use render::svg::{
    SvgElementRender as Svg, SvgListItemRender, SvgNodes, SvgRender, SvgStaticNodes,
};
pub use render::{
    html::predefined_attribute_types::*,
    html::HtmlElementRender as Element,
    html::{ListItemRender, Nodes, Render, StaticNodes, StaticRender},
    ListElementCreation,
};

// TODO selectively export event traits only?
pub use events::*;
pub use fetch::{FetchError, ResponsedError};
pub use future::Future;
pub use routing::{Router, Routes};
pub use utils::*;

pub use http;
pub use web_sys;

pub use wasm_bindgen::JsValue;
pub use wasm_bindgen_futures::JsFuture;

pub mod prelude {
    pub use crate::application::Application;
    pub use crate::callback::{
        Callback as TraitCallback, CallbackArg as TraitCallbackArg,
        CallbackOnce as TraitCallbackOne, CallbackOnceArg as TraitCallbackOnceArg,
    };
    pub use crate::component::{AsChildComp, Component};
    pub use crate::render::base::MethodsForEvents;
    pub use crate::render::html::{
        HamsForAmbiguousNames, HamsForDistinctNames, MethodsForSelectedValueSelectedIndex,
        HamsHandMade, HamsWithPredefinedValues, HemsForAmbiguousNames, HemsForDistinctNames,
        HemsForList, HemsForPartialList, HemsHamsAmbiguous, HemsHandMade,
        MethodsForHtmlElementContent,
    };

    #[cfg(feature = "keyed-list")]
    pub use crate::render::html::HemsForKeyedList;

    pub use crate::fetch::{FetchOptionsSetter, RawDataMode};
    #[cfg(feature = "svg")]
    pub use crate::render::svg::{
        MethodsForSvgElementContent, SamsForDistinctNames, SamsHandMade, SemsForDistinctNames,
        SemsForList, SemsForPartialList, SemsHandMade,
    };

    #[cfg(all(feature = "keyed-list", feature = "svg"))]
    pub use crate::render::svg::SemsForKeyedList;

    pub use crate::routing::Routes;
    pub use wasm_bindgen;
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::{JsCast, UnwrapThrowExt};
}

pub type Callback = Box<dyn callback::Callback>;
pub type CallbackArg<A> = Box<dyn callback::CallbackArg<A>>;

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

    pub fn set<C: component::Component>(
        &self,
        element: &crate::render::html::HtmlElementRender<C>,
    ) {
        let e = element.ws_element().unchecked_into::<T>();
        *self.0.borrow_mut() = Some(e);
    }

    pub fn execute(&self, f: impl FnOnce(&T)) {
        if let Some(t) = self.get().as_ref() {
            f(t);
        }
    }
}
