use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use crate::component::{Comp, Component};

struct SpairRouter {
    router: Box<dyn std::any::Any>,
    current_url: Rc<RefCell<Option<String>>>,
    _pop_state_closure: Option<wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)>>,
}

impl SpairRouter {
    fn execute_routing<R: Router>(&self) {
        let location = match get_new_location(&self.current_url) {
            Some(location) => location,
            None => return,
        };
        if let Some(router) = self.router.downcast_ref::<R>() {
            router.routing(location);
        }
    }
}

thread_local! {
    static ROUTER: Rc<RefCell<SpairRouter>> = Rc::new(RefCell::new(SpairRouter{
        router: Box::new(()),
        current_url: Rc::new(RefCell::new(None)),
        _pop_state_closure: None,
    }));
}

pub trait Router: std::any::Any {
    fn routing(&self, location: web_sys::Location);
}

pub trait Routes {
    type Router: Router;
    fn url(&self) -> String;
    fn update_address_bar(&self) {
        crate::utils::window()
            .history()
            .expect_throw("Unable to get history")
            .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&self.url()))
            .expect_throw("Error on push_state_with_url");
    }

    fn execute_routing(&self) {
        self.update_address_bar();
        crate::component::update_component(self::execute_routing::<Self::Router>);
    }
}

impl Router for () {
    fn routing(&self, _: web_sys::Location) {}
}

impl Routes for () {
    type Router = ();
    fn url(&self) -> String {
        String::new()
    }
}

pub fn set_router<R: Router>(r: R) {
    ROUTER.with(|router| {
        if let Ok(mut router) = router.try_borrow_mut() {
            router.router = Box::new(r);
            router._pop_state_closure = Some(register_pop_state_event::<R>());
        }
    });
}

pub fn execute_routing<R: Router>() {
    ROUTER.with(|router| {
        if let Ok(router) = router.try_borrow() {
            router.execute_routing::<R>();
        }
    });
}

pub fn register_routing_callback<C: Component>(comp: &Comp<C>) {
    modify_router::<C, _>(|router| C::register_routing_callback(router, comp))
}

pub fn remove_routing_callback<C: Component>() {
    modify_router::<C, _>(C::remove_routing_callback)
}

pub fn modify_router<C: Component, F: FnOnce(&mut <<C as Component>::Routes as Routes>::Router)>(
    f: F,
) {
    ROUTER.with(|router| {
        if let Ok(mut router) = router.try_borrow_mut() {
            if let Some(router) = router
                .router
                .downcast_mut::<<<C as Component>::Routes as Routes>::Router>()
            {
                f(router);
            }
        }
    })
}

fn register_pop_state_event<R: Router>(
) -> wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)> {
    let closure = move |_: web_sys::PopStateEvent| {
        ROUTER.with(|router| {
            if let Ok(router) = router.try_borrow() {
                router.execute_routing::<R>();
            }
        })
    };

    let closure = wasm_bindgen::closure::Closure::wrap(
        Box::new(closure) as Box<dyn Fn(web_sys::PopStateEvent)>
    );

    crate::utils::register_event_listener_on_window("popstate", closure.as_ref().unchecked_ref());

    closure
}

fn get_new_location(current_url: &Rc<RefCell<Option<String>>>) -> Option<web_sys::Location> {
    let location = crate::utils::window().location();
    let new_url = location
        .href()
        .expect_throw("Unable to get window.location.href");
    let mut current_url = current_url
        .try_borrow_mut()
        .expect_throw("Multiple mutable borrow on current_url");
    let new_url = Some(new_url);
    if *current_url != new_url {
        *current_url = new_url;
        Some(location)
    } else {
        None
    }
}
