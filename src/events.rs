use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub trait Listener {
    fn remove_listener_from_element(&mut self);
}

macro_rules! create_methods_for_event_trait {
    ($($method_name:ident $EventName:ident,)+) => {
        $(
            fn $method_name<F>(mut self, f: F) -> Self
            where F: $EventName
            {
                let er = self.element_updater_mut();
                if er.require_set_listener() {
                    let listener = $EventName::on(f, er.element().ws_element().ws_event_target());
                    er.store_listener(listener);
                }
                self
            }
        )+
    }
}

macro_rules! create_events {
    ($(
        $EventType:ident $EventListener:ident {
            $($EventName:ident => $event_name:literal $event_method_name:ident,)+
        }
    )+) => {
        pub trait MethodsForEvents<'er, C: crate::component::Component>: Sized + crate::render::base::ElementUpdaterMut<'er, C> {
            $(
                create_methods_for_event_trait! {
                    $($event_method_name $EventName,)+
                }
            )+
        }
        $(
            pub struct $EventType(web_sys::$EventType);
            impl $EventType {
                pub fn raw_event_type(&self) -> &web_sys::$EventType {
                    &self.0
                }

                pub fn target(&self) -> crate::element::EventTarget {
                    crate::element::EventTarget(self.0.target())
                }

                pub fn current_target(&self) -> crate::element::EventTarget {
                    crate::element::EventTarget(self.0.current_target())
                }
            }

            pub struct $EventListener {
                event_name: &'static str,
                event_target: web_sys::EventTarget,
                closure: Closure<dyn Fn(web_sys::$EventType)>,
            }
            impl $EventListener {
                fn new(event_name: &'static str, event_target: &web_sys::EventTarget, closure: Closure<dyn Fn(web_sys::$EventType)>) -> Self {
                    event_target.add_event_listener_with_callback(
                        wasm_bindgen::intern(event_name),
                        closure.as_ref().unchecked_ref()
                    ).expect_throw("Expect event register to be successful");
                    Self {
                        event_name,
                        event_target: event_target.clone(),
                        closure,
                    }
                }
            }

            impl Listener for $EventListener {
                fn remove_listener_from_element(&mut self) {
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
                    T: 'static + crate::callback::CallbackArg<$EventType>,
                {
                    fn on(self, target: &web_sys::EventTarget) -> Box<dyn Listener> {
                        let closure = move |event: web_sys::$EventType| self.call($EventType(event));
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
    // ClipboardEvent ClipboardEventListener {
    // web_sys requires to compile with a flag to work with clipboard api,
    // these apis are moved to the Event section to use the basic functionalities provide by Event
    // without having to compile with the flag.
    // }
    Event EventListener {
        Copy => "copy" on_copy,
        Cut => "cut" on_cut,
        Paste => "paste" on_paste,

        Change => "change" on_change,
        Ended => "ended" on_ended,
        PointerLockChange => "pointerlockchange" on_pointer_lock_change,
        PointerLockError => "pointerlockerror" on_pointer_lock_error,
        Reset => "reset" on_reset,
        Submit => "submit" on_submit,

    }
    FocusEvent FocusEventListener {
        Blur => "blur" on_blur,
        Focus => "focus" on_focus,
    }
    InputEvent InputEventListener {
        Input => "input" on_input,
    }
    KeyboardEvent KeyboardEventListener {
        KeyDown => "keydown" on_key_down,
        KeyPress => "keypress" on_key_press,
        KeyUp => "keyup" on_key_up,
    }
    MouseEvent MouseEventListener {
        AuxClick => "auxclick" on_aux_click,
        Click => "click" on_click,
        ContextMenu => "contextmenu" on_context_menu,
        DblClick => "dblclick" on_double_click,
        MouseDown => "mousedown" on_mouse_down,
        MouseEnter => "mouseenter" on_mouse_enter,
        MouseLeave => "mouseleave" on_mouse_leave,
        MouseMove => "mousemove" on_mouse_move,
        MouseOut => "mouseout" on_mouse_out,
        MouseOver => "mouseover" on_mouse_over,
        MouseUp => "mouseup" on_mouse_up,
    }
    WheelEvent WheelEventListener {
        Wheel => "wheel" on_wheel,
    }
}
