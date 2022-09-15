use super::BaseElementRenderMut;
use crate::component::Component;

macro_rules! create_methods_for_events {
    ($($method_name:ident $EventName:ident,)+) => {
        $(
            fn $method_name<F>(mut self, f: F) -> Self
            where F: crate::events::$EventName
            {
                let er = self.element_render_mut();
                if er.require_set_listener() {
                    let listener = crate::events::$EventName::on(f, er.element().ws_element().ws_event_target());
                    er.store_listener(listener);
                }
                self
            }
        )+
    }
}

pub trait MethodsForEvents<C: Component>: Sized + BaseElementRenderMut<C> {
    create_methods_for_events! {
        on_focus Focus,
        on_blur Blur,

        on_aux_click AuxClick,
        on_click Click,
        on_double_click DoubleClick,
        on_mouse_enter MouseEnter,
        on_mouse_over MouseOver,
        on_mouse_move MouseMove,
        on_mouse_down MouseDown,
        on_mouse_up MouseUp,
        on_mouse_leave MouseLeave,
        on_mouse_out MouseOut,
        on_context_menu ContextMenu,

        on_wheel Wheel,
        on_select UiSelect,

        on_input Input,

        on_key_down KeyDown,
        on_key_press KeyPress,
        on_key_up KeyUp,

        on_change Change,
        on_reset Reset,
        on_submit Submit,
        on_pointer_lock_change PointerLockChange,
        on_pointer_lock_error PointerLockError,

        on_ended Ended,
    }
}
