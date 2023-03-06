pub enum AttributeValue {
    EventListener(Option<Box<dyn crate::events::Listener>>),
    String(Option<String>),
    Bool(bool),
    I32(i32),
    U32(u32),
    F64(f64),
}

impl Clone for AttributeValue {
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

impl std::fmt::Debug for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::EventListener(_) => f.write_str("EventListener(...)"),
            Self::String(value) => value.fmt(f),
            Self::Bool(value) => value.fmt(f),
            Self::I32(value) => value.fmt(f),
            Self::U32(value) => value.fmt(f),
            Self::F64(value) => value.fmt(f),
        }
    }
}

// Never apply to properties or classes....
pub trait AttributeValueAsString: Into<AttributeValue> {
    fn set(&self, name: &str, ws: &super::WsElement);
    fn update(self, index: usize, list: &mut AttributeValueList, name: &str, ws: &super::WsElement);
}

macro_rules! impl_attribute_value_changed {
    ($($Var:ident($type:ty),)+) => {
        $(
            impl From<$type> for AttributeValue {
                fn from(value: $type) -> Self {
                    Self::$Var(value)
                }
            }

            impl AttributeValueAsString for $type {
                fn set(&self, name: &str, ws: &super::WsElement) {
                    ws.set_str_attribute(name, &self.to_string());
                }

                fn update(self, index: usize, list: &mut AttributeValueList, name: &str, ws: &super::WsElement) {
                    match list.0.get_mut(index) {
                        None => {
                            list.0.push(self.into());
                            self.set(name, ws);
                        }
                        Some(current_value) => match current_value {
                            AttributeValue::$Var(current_value) if *current_value != self => {
                                *current_value = self;
                                self.set(name, ws);
                            }
                            AttributeValue::$Var(_) => {}
                            _ => panic!("Spair's internal error, expected an AttributeValue::{}?", stringify!($Var)),
                        }
                    }
                }
            }
        )+
    };
}

impl_attribute_value_changed! {
    U32(u32),
    F64(f64),
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl AttributeValueAsString for bool {
    fn set(&self, name: &str, ws: &super::WsElement) {
        ws.set_bool_attribute(name, *self);
    }

    fn update(
        self,
        index: usize,
        list: &mut AttributeValueList,
        name: &str,
        ws: &super::WsElement,
    ) {
        if list.bool_value_change(index, self) {
            self.set(name, ws);
        }
    }
}

impl From<i32> for AttributeValue {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl AttributeValueAsString for i32 {
    fn set(&self, name: &str, ws: &super::WsElement) {
        ws.set_str_attribute(name, &self.to_string());
    }

    fn update(
        self,
        index: usize,
        list: &mut AttributeValueList,
        name: &str,
        ws: &super::WsElement,
    ) {
        if list.i32_value_change(index, self) {
            self.set(name, ws);
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(Some(value.to_string()))
    }
}

impl AttributeValueAsString for &str {
    fn set(&self, name: &str, ws: &super::WsElement) {
        ws.set_str_attribute(name, self);
    }

    fn update(
        self,
        index: usize,
        list: &mut AttributeValueList,
        name: &str,
        ws: &super::WsElement,
    ) {
        if list.option_str_value_change(index, Some(self)).0 {
            self.set(name, ws);
        }
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::String(Some(value))
    }
}

// String has special impl here because I want take ownership of self
// if users give Spair a String. The reason is to avoid a `.to_string()`
impl AttributeValueAsString for String {
    fn set(&self, name: &str, ws: &super::WsElement) {
        ws.set_str_attribute(name, self);
    }

    fn update(
        self,
        index: usize,
        list: &mut AttributeValueList,
        name: &str,
        ws: &super::WsElement,
    ) {
        match list.0.get_mut(index) {
            None => {
                self.set(name, ws);
                list.0.push(self.into());
            }
            Some(current_value) => match current_value {
                AttributeValue::String(current_value)
                    if current_value.as_deref() != Some(self.as_str()) =>
                {
                    self.set(name, ws);
                    // avoid to_string here, if it's a `&str` then `.to_string()` is required
                    *current_value = Some(self);
                }
                AttributeValue::String(_) => {}
                _ => panic!("Spair's internal error, expected an AttributeValue::String?"),
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AttributeValueList(Vec<AttributeValue>);

impl AttributeValueList {
    pub fn store_listener(&mut self, index: usize, listener: Box<dyn crate::events::Listener>) {
        if index < self.0.len() {
            let mut listener = AttributeValue::EventListener(Some(listener));
            std::mem::swap(&mut self.0[index], &mut listener);
            if let AttributeValue::EventListener(Some(mut listener)) = listener {
                listener.remove_listener_from_element();
            }
        } else {
            self.0.push(AttributeValue::EventListener(Some(listener)));
        }
    }

    pub fn bool_value_change(&mut self, index: usize, value: bool) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::Bool(value));
                true
            }
            Some(a) => match a {
                AttributeValue::Bool(old_value) if value == *old_value => false,
                AttributeValue::Bool(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::Bool?"),
            },
        }
    }

    pub fn i32_value_change(&mut self, index: usize, value: i32) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::I32(value));
                true
            }
            Some(a) => match a {
                AttributeValue::I32(old_value) if value == *old_value => false,
                AttributeValue::I32(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::I32?"),
            },
        }
    }

    pub fn option_str_value_change(
        &mut self,
        index: usize,
        value: Option<&str>,
    ) -> (bool, Option<String>) {
        match self.0.get_mut(index) {
            None => {
                self.0
                    .push(AttributeValue::String(value.map(ToString::to_string)));
                (true, None)
            }
            Some(a) => match a {
                AttributeValue::String(old_value) if value == old_value.as_deref() => (false, None),
                AttributeValue::String(old_value) => {
                    let mut new_value = value.map(ToString::to_string);
                    std::mem::swap(old_value, &mut new_value);
                    (true, new_value)
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::String?"),
            },
        }
    }

    pub fn option_string_value_change(
        &mut self,
        index: usize,
        mut value: Option<String>,
    ) -> (bool, Option<String>) {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::String(value));
                (true, None)
            }
            Some(a) => match a {
                AttributeValue::String(old_value) if value == *old_value => (false, None),
                AttributeValue::String(old_value) => {
                    std::mem::swap(old_value, &mut value);
                    (true, value)
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::String?"),
            },
        }
    }
}
