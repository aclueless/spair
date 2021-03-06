use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub trait Listener {}

macro_rules! create_event_wrappers {
    ($($EventType:ident)+) => {
        $(
            pub struct $EventType(web_sys::$EventType);
            impl $EventType {
                pub fn raw(&self) -> &web_sys::$EventType {
                    &self.0
                }

                pub fn target(&self) -> Option<web_sys::EventTarget> {
                    self.0.target()
                }

                pub fn target_as<T: JsCast>(&self) -> Option<T> {
                    self.0.target().and_then(|et| et.dyn_into().ok())
                }
            }
        )+
    }
}

create_event_wrappers! {
    FocusEvent
    MouseEvent
    WheelEvent
    UiEvent
    InputEvent
    KeyboardEvent
    Event
}

impl InputEvent {
    pub fn target_as_input_element(&self) -> Option<web_sys::HtmlInputElement> {
        self.target_as()
    }
}

impl Event {
    pub fn target_as_select_element(&self) -> Option<web_sys::HtmlSelectElement> {
        self.target_as()
    }
}

macro_rules! create_events {
    ($($EventType:ident $EventListener:ident { $($EventName:ident => $event_name:literal,)+ })+) => {
        $(
            pub struct $EventListener {
                event_name: &'static str,
                event_target: web_sys::EventTarget,
                closure: Closure<dyn Fn(web_sys::$EventType)>,
            }
            impl $EventListener {
                // TODO: remove duplicated code here
                fn new(event_name: &'static str, event_target: &web_sys::EventTarget, closure: Closure<dyn Fn(web_sys::$EventType)>) -> Self {
                    event_target.add_event_listener_with_callback(
                        event_name,
                        closure.as_ref().unchecked_ref()
                    ).expect_throw("Expect event register to be successful");
                    Self {
                        event_name,
                        event_target: event_target.clone(),
                        closure,
                    }
                }
            }

            impl Listener for $EventListener {}

            impl Drop for $EventListener {
                #[inline]
                fn drop(&mut self) {
                    self.event_target
                        .remove_event_listener_with_callback(
                            self.event_name,
                            self.closure.as_ref().unchecked_ref()
                        ).expect_throw("Expect event removal to be successful");
                }
            }
            $(
                #[doc = "Help creating "]
                #[doc = $event_name]
                #[doc = " event listener"]
                pub trait $EventName {
                    fn on(self, node: &web_sys::EventTarget) -> Box<dyn Listener>;
                    fn on_window(self) -> Box<dyn Listener>;
                }

                impl<T> $EventName for T
                where
                    T: 'static + Fn($EventType),
                {
                    fn on(self, target: &web_sys::EventTarget) -> Box<dyn Listener> {
                        let closure = move |event: web_sys::$EventType| self($EventType(event));
                        let closure = Closure::wrap(Box::new(closure) as Box<dyn Fn(web_sys::$EventType)>);
                        Box::new($EventListener::new($event_name, target, closure))
                    }

                    fn on_window(self) -> Box<dyn Listener> {
                        $EventName::on(self, crate::utils::window().as_ref())
                    }
                }
            )+
        )+
    };
}

create_events! {
    FocusEvent FocusEventListener {
        Focus => "focus",
        Blur => "blur",
    }
    MouseEvent MouseEventListener {
        AuxClick => "auxclick",
        Click => "click",
        DblClick => "dblclick",
        DoubleClick => "dblclick",
        MouseEnter => "mouseenter",
        MouseOver => "mouseover",
        MouseMove => "mousemove",
        MouseDown => "mousedown",
        MouseUp => "mouseup",
        MouseLeave => "mouseleave",
        MouseOut => "mouseout",
        ContextMenu => "contextmenu",
    }
    WheelEvent WheelEventListener {
        Wheel => "wheel",
    }
    UiEvent UiEventListener {
        UiSelect => "select",
    }
    InputEvent InputEventListener {
        Input => "input",
    }
    KeyboardEvent KeyboardEventListener {
        KeyDown => "keydown",
        KeyPress => "keypress",
        KeyUp => "keyup",
    }
    Event EventListener {
        Change => "change",
        Reset => "reset",
        Submit => "submit",
        PointerLockChange => "pointerlockchange",
        PointerLockError => "pointerlockerror",

        Ended => "ended",
    }
}
