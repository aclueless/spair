use js_sys::Function;
use wasm_bindgen::{closure::Closure, JsCast};

pub trait EventListener {
    fn js_function(&self) -> &Function;
}

macro_rules! impl_event_listener_trait {
    ($($EventArg:ident)+) => {
        $(
        impl EventListener for Closure<dyn Fn(web_sys::$EventArg)> {
            fn js_function(&self) -> &Function {
                self.as_ref().unchecked_ref()
            }
        }
        )+
    };
}

impl_event_listener_trait! {
    Event
    MouseEvent
    FocusEvent
    InputEvent
    KeyboardEvent
    WheelEvent
    PopStateEvent
}
