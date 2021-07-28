use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

struct SpairRouter {
    router: Box<dyn std::any::Any>,
    current_url: Rc<RefCell<Option<String>>>,
    _pop_state_closure: wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)>,
}

impl SpairRouter {
    //fn execute_routing<C: crate::component::Component>(&self) {
    fn execute_routing(&self) {
        let location = match get_new_location(&self.current_url) {
            Some(location) => location,
            None => return,
        };
        if let Some(router) = self.router.downcast_ref::<Box<dyn Router>>() {
            router.routing(location);
        }
    }
}

thread_local! {
    static ROUTER: Rc<RefCell<SpairRouter>> = Rc::new(RefCell::new(SpairRouter{
        router: Box::new(()),
        current_url: Rc::new(RefCell::new(None)),
        _pop_state_closure: register_pop_state_event(),
    }));
}

fn register_pop_state_event() -> wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)> {
    let closure = move |_: web_sys::PopStateEvent| {
        ROUTER.with(|router| {
            if let Ok(router) = router.try_borrow() {
                router.execute_routing();
            }
        })
    };

    let closure = wasm_bindgen::closure::Closure::wrap(
        Box::new(closure) as Box<dyn Fn(web_sys::PopStateEvent)>
    );

    register_event_listener_on_window("popstate", closure.as_ref().unchecked_ref());

    closure
}

fn register_event_listener_on_window(event: &str, listener: &js_sys::Function) {
    let window = crate::utils::window();
    let window: &web_sys::EventTarget = window.as_ref();
    window
        .add_event_listener_with_callback(event, listener)
        .expect_throw("Unable to register event listener on window");
}

pub trait Router {
    fn routing(&self, location: web_sys::Location);
}

pub trait Routes {
    type Router: Router;
    /// Just help creating a `ghost router` for application that has `type Routes = ();`
    /// You never need to override this method. But you should override `Application::init_router`
    /// to provide your actual Router instance, if not your app will fail immediately.
    /// This method was put here but not in `Router` to allow making `Router` a trait object.
    fn unit_router() -> Self::Router {
        unreachable!(
            "You must implement method `Application::init_router` and provide the actual router instance"
        )
    }

    fn url(&self) -> String;
}

impl Router for () {
    fn routing(&self, _: web_sys::Location) {}
}

impl Routes for () {
    type Router = ();
    fn unit_router() -> Self {}
    fn url(&self) -> String {
        String::new()
    }
}

pub fn set_router<R: 'static + Router>(r: R) {
    ROUTER.with(|router| {
        if let Ok(mut router) = router.try_borrow_mut() {
            router.router = Box::new(r);
            router.execute_routing();
        }
    });
}

pub fn register_routing_callback<C: crate::component::Component>(comp: &crate::component::Comp<C>) {
    ROUTER.with(|router| {
        if let Ok(mut router) = router.try_borrow_mut() {
            if let Some(router) = router
                .router
                .downcast_mut::<Box<<<C as crate::component::Component>::Routes2 as Routes>::Router>>(
            ) {
                C::register_routing_callback(router, comp);
            }
        }
    })
}

pub fn remove_routing_callback<C: crate::component::Component>() {
    ROUTER.with(|router| {
        if let Ok(mut router) = router.try_borrow_mut() {
            if let Some(router) = router
                .router
                .downcast_mut::<Box<<<C as crate::component::Component>::Routes2 as Routes>::Router>>(
            ) {
                C::remove_routing_callback(router);
            }
        }
    })
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
