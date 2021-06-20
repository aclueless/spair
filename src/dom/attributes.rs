use wasm_bindgen::UnwrapThrowExt;

enum Attribute {
    EventListener(Option<Box<dyn crate::events::Listener>>),
    String(String),
    Bool(bool),
    I32(i32),
    U32(u32),
    F64(f64),
}

impl Clone for Attribute {
    fn clone(&self) -> Self {
        match self {
            Self::EventListener(_) => Self::EventListener(None),
            Self::String(v) => Self::String(v.clone()),
            Self::Bool(v) => Self::Bool(*v),
            Self::I32(v) => Self::I32(*v),
            Self::U32(v) => Self::U32(*v),
            Self::F64(v) => Self::F64(*v),
        }
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::EventListener(_) => f.write_str("EventListener(...)"),
            Self::Bool(value) => value.fmt(f),
            Self::String(value) => value.fmt(f),
            Self::I32(value) => value.fmt(f),
            Self::U32(value) => value.fmt(f),
            Self::F64(value) => value.fmt(f),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AttributeList(Vec<Attribute>);

impl AttributeList {
    pub(super) fn store_listener(
        &mut self,
        index: usize,
        listener: Box<dyn crate::events::Listener>,
    ) {
        if index < self.0.len() {
            self.0[index] = Attribute::EventListener(Some(listener));
        } else {
            self.0.push(Attribute::EventListener(Some(listener)));
        }
    }

    pub(super) fn check_bool_attribute(&mut self, index: usize, value: bool) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::Bool(value));
                true
            }
            Some(a) => match a {
                Attribute::Bool(old_value) if value == *old_value => false,
                Attribute::Bool(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Why not an Attribute::Bool?"),
            },
        }
    }

    pub(super) fn check_i32_attribute(&mut self, index: usize, value: i32) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::I32(value));
                true
            }
            Some(a) => match a {
                Attribute::I32(old_value) if value == *old_value => false,
                Attribute::I32(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Why not an Attribute::I32?"),
            },
        }
    }

    pub(super) fn check_u32_attribute(&mut self, index: usize, value: u32) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::U32(value));
                true
            }
            Some(a) => match a {
                Attribute::U32(old_value) if value == *old_value => false,
                Attribute::U32(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Why not an Attribute::U32?"),
            },
        }
    }

    pub(super) fn check_f64_attribute(&mut self, index: usize, value: f64) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::F64(value));
                true
            }
            Some(a) => match a {
                Attribute::F64(old_value) if (value - *old_value).abs() < std::f64::EPSILON => {
                    false
                }
                Attribute::F64(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Why not an Attribute::F64?"),
            },
        }
    }

    pub(super) fn check_str_attribute(&mut self, index: usize, value: &str) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::String(value.to_string()));
                true
            }
            Some(a) => match a {
                Attribute::String(old_value) if value == *old_value => false,
                Attribute::String(old_value) => {
                    *old_value = value.to_string();
                    true
                }
                _ => panic!("Why not an Attribute::String?"),
            },
        }
    }

    pub(super) fn check_str_attribute_and_return_old_value(
        &mut self,
        index: usize,
        value: &str,
    ) -> (bool, Option<String>) {
        match self.0.get_mut(index) {
            None => {
                self.0.push(Attribute::String(value.to_string()));
                (true, None)
            }
            Some(a) => match a {
                Attribute::String(old_value) if value == *old_value => (false, None),
                Attribute::String(old_value) => {
                    let mut value = value.to_string();
                    std::mem::swap(&mut value, old_value);
                    (true, Some(value))
                }
                _ => panic!("Why not an Attribute::String?"),
            },
        }
    }
}

pub trait AttributeSetter {
    fn ws_html_element(&self) -> &web_sys::HtmlElement;
    fn ws_element(&self) -> &web_sys::Element;
    fn element_type(&self) -> crate::dom::ElementType;
    fn require_set_listener(&mut self) -> bool;
    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>);

    // Check if the attribute need to be set (and store the new value for the next check)
    fn check_bool_attribute(&mut self, value: bool) -> bool;
    fn check_str_attribute(&mut self, value: &str) -> bool;
    fn check_i32_attribute(&mut self, value: i32) -> bool;
    fn check_u32_attribute(&mut self, value: u32) -> bool;
    fn check_f64_attribute(&mut self, value: f64) -> bool;
    fn check_str_attribute_and_return_old_value(&mut self, value: &str) -> (bool, Option<String>);

    fn set_bool_attribute(&mut self, name: &str, value: bool) {
        if self.check_bool_attribute(value) {
            if value {
                self.ws_element()
                    .set_attribute(name, "")
                    .expect_throw("Unable to set bool attribute");
            } else {
                self.ws_element()
                    .remove_attribute(name)
                    .expect_throw("Unable to remove bool attribute");
            }
        }
    }

    fn set_str_attribute(&mut self, name: &str, value: &str) {
        if self.check_str_attribute(value) {
            self.ws_element()
                .set_attribute(name, value)
                .expect_throw("Unable to set string attribute");
        }
    }

    fn set_i32_attribute(&mut self, name: &str, value: i32) {
        if self.check_i32_attribute(value) {
            self.ws_element()
                .set_attribute(name, &value.to_string())
                .expect_throw("Unable to set string attribute");
        }
    }

    fn set_u32_attribute(&mut self, name: &str, value: u32) {
        if self.check_u32_attribute(value) {
            self.ws_element()
                .set_attribute(name, &value.to_string())
                .expect_throw("Unable to set string attribute");
        }
    }

    fn set_f64_attribute(&mut self, name: &str, value: f64) {
        if self.check_f64_attribute(value) {
            self.ws_element()
                .set_attribute(name, &value.to_string())
                .expect_throw("Unable to set string attribute");
        }
    }
}

macro_rules! create_methods_for_events {
    ($($method_name:ident $EventName:ident,)+) => {
        $(
            fn $method_name<F>(mut self, f: F) -> Self
            where F: crate::events::$EventName
            {
                if self.require_set_listener() {
                    let listener = crate::events::$EventName::on(f, self.ws_element().as_ref());
                    self.store_listener(listener);
                }
                self
            }
        )+
    }
}

macro_rules! create_methods_for_attributes {
    (
        $(
            $attribute_type:ident $method_name:ident $($attribute_name:literal)?
        )+
    ) => {
        $(
            create_methods_for_attributes! {
                @each
                $method_name $($attribute_name)? => $attribute_type
            }
        )+
    };
    (@each $method_name:ident => $attribute_type:ident) => {
        create_methods_for_attributes! {
            @each
            $method_name stringify!($method_name) => $attribute_type
        }
    };
    (@each $method_name:ident $attribute_name:expr => bool) => {
        create_methods_for_attributes! {
            @create
            $method_name $attribute_name => bool => set_bool_attribute
        }
    };
    (@each $method_name:ident $attribute_name:expr => u32) => {
        create_methods_for_attributes! {
            @create
            $method_name $attribute_name => u32 => set_u32_attribute
        }
    };
    (@each $method_name:ident $attribute_name:expr => i32) => {
        create_methods_for_attributes! {
            @create
            $method_name $attribute_name => i32 => set_i32_attribute
        }
    };
    (@each $method_name:ident $attribute_name:expr => f64) => {
        create_methods_for_attributes! {
            @create
            $method_name $attribute_name => f64 => set_f64_attribute
        }
    };
    (@each $method_name:ident $attribute_name:expr => str) => {
        create_methods_for_attributes! {
            @create
            $method_name $attribute_name => &str => set_str_attribute
        }
    };
    (@each $method_name:ident $attribute_name:expr => AsStr) => {
        fn $method_name(mut self, value: impl crate::dom::AsStr) -> Self {
            self.set_str_attribute($attribute_name, value.as_str());
            self
        }
    };
    (@create $method_name:ident $attribute_name:expr => $attribute_type:ty => $shared_method_name:ident) => {
        #[allow(clippy::wrong_self_convention)]
        fn $method_name(mut self, value: $attribute_type) -> Self {
            self.$shared_method_name($attribute_name, value);
            self
        }
    };
}

pub trait EventSetter: Sized + AttributeSetter {
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
