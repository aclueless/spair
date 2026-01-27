use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt, prelude::Closure};
use web_sys::{HtmlAnchorElement, HtmlAreaElement, Location, MouseEvent};

use crate::{
    component::{Comp, Component},
    dom::WsElement,
    events::EventListener,
    helper,
};

pub trait Route: Sized {
    fn from_location(location: &Location) -> Self;
    fn url(&self) -> String;
}

impl Route for () {
    fn from_location(_location: &Location) -> Self {}
    fn url(&self) -> String {
        String::new()
    }
}

pub fn setup_routing<C: 'static + Component, R: 'static + Route>(
    set_route: impl Fn(&mut C, R) + 'static,
    comp: Comp<C>,
) {
    LAST_HREF.set(get_current_location().href().unwrap_throw());

    let set_route = comp.callback_arg(set_route);
    let do_routing = move || {
        let current_location = get_current_location();
        let current_href = current_location.href().unwrap_throw();
        if LAST_HREF.with(|last_href| current_href.as_str() == *last_href.borrow()) {
            return;
        }
        let route = R::from_location(&current_location);
        LAST_HREF.set(current_href);
        set_route.call(route);
    };
    let do_routing = Rc::new(do_routing);
    let clone_routing = do_routing.clone();
    let closure = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |me: web_sys::MouseEvent| {
        me.prevent_default();
        let Some(element) = me.current_target() else {
            return;
        };
        let href = if let Some(a) = element.dyn_ref::<HtmlAnchorElement>() {
            a.href()
        } else if let Some(area) = element.dyn_ref::<HtmlAreaElement>() {
            area.href()
        } else {
            return;
        };
        helper::WINDOW.with(|window| {
            if let Err(e) = window.history().unwrap_throw().push_state_with_url(
                &JsValue::NULL,
                "",
                Some(href.as_str()),
            ) {
                log::error!("Error on push_state_with_url: {e:?}");
            }
        });
        do_routing()
    });
    if HREF_ELEMENT_CLICK_HANDLER
        .with(|value| value.set(closure))
        .is_err()
    {
        log::error!("Error on storing HREF_ELEMENT_CLICK_HANDLER, why the value is already set?");
    };
    let do_routing = clone_routing;
    let closure = Closure::<dyn Fn(web_sys::PopStateEvent)>::new(move |_| do_routing());
    helper::WINDOW.with(|window| {
        if let Err(e) = window.add_event_listener_with_callback("popstate", closure.js_function()) {
            log::error!("Error on adding window-popstate-event listener for routing: {e:?}");
        };
    });
    closure.forget();
}

pub(crate) fn add_routing_handler(target: &WsElement) {
    HREF_ELEMENT_CLICK_HANDLER.with(|handler| {
        if let Some(handler) = handler.get() {
            target.add_event_listener("click", handler);
        } else {
            log::warn!("HREF_ELEMENT_CLICK_HANDLER not found. Routing will not work.")
        }
    })
}

thread_local! {
    static LAST_HREF: RefCell<String> = const{RefCell::new(String::new())};
    static HREF_ELEMENT_CLICK_HANDLER: OnceCell<Closure<dyn Fn(MouseEvent)>> = OnceCell::new();
}

pub fn get_current_location() -> Location {
    helper::WINDOW.with(|window| window.location())
}
