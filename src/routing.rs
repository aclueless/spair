use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub trait Routes<C>
where
    C: crate::component::Component,
{
    fn url(&self) -> String;
    fn routing(location: web_sys::Location, comp: &crate::component::Comp<C>);
    fn router(comp: &crate::component::Comp<C>) -> Option<Router> {
        Some(Router::new(comp))
    }
}

impl<C> Routes<C> for ()
where
    C: crate::component::Component,
{
    fn url(&self) -> String {
        String::new()
    }
    fn routing(_: web_sys::Location, _: &crate::component::Comp<C>) {}
    fn router(_: &crate::component::Comp<C>) -> Option<Router> {
        None
    }
}

pub struct Router {
    _current_url: Rc<RefCell<String>>,
    _pop_state_closure: wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)>,
}

impl Router {
    fn new<C>(comp: &crate::component::Comp<C>) -> Self
    where
        C: crate::component::Component,
    {
        let _current_url = Rc::new(RefCell::new(String::new()));
        let location =
            get_new_location(&_current_url).expect_throw("Why the first url not invalid?");
        C::Routes::routing(location, comp);

        let _pop_state_closure = register_pop_state_event(_current_url.clone(), comp.clone());

        Self {
            _pop_state_closure,
            _current_url,
        }
    }
}

fn get_new_location(current_url: &Rc<RefCell<String>>) -> Option<web_sys::Location> {
    let location = crate::utils::window().location();
    let new_url = location
        .href()
        .expect_throw("Unable to get window.location.href");
    let mut current_url = current_url
        .try_borrow_mut()
        .expect_throw("Multiple mutable borrow on current_url");
    if *current_url != new_url {
        *current_url = new_url;
        Some(location)
    } else {
        None
    }
}

fn register_event_listener_on_window(event: &str, listener: &js_sys::Function) {
    let window = crate::utils::window();
    let window: &web_sys::EventTarget = window.as_ref();
    window
        .add_event_listener_with_callback(event, listener)
        .expect_throw("Unable to register event listener on window");
}

fn register_pop_state_event<C>(
    current_url: Rc<RefCell<String>>,
    comp: crate::component::Comp<C>,
) -> wasm_bindgen::closure::Closure<dyn Fn(web_sys::PopStateEvent)>
where
    C: crate::component::Component,
{
    let closure = move |_: web_sys::PopStateEvent| {
        if let Some(location) = get_new_location(&current_url) {
            C::Routes::routing(location, &comp);
        }
    };

    let closure = wasm_bindgen::closure::Closure::wrap(
        Box::new(closure) as Box<dyn Fn(web_sys::PopStateEvent)>
    );

    register_event_listener_on_window("popstate", closure.as_ref().unchecked_ref());

    closure
}
