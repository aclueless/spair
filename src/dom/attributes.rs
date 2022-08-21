enum AttributeValue {
    EventListener(Option<Box<dyn crate::events::Listener>>),
    String(String),
    SelectedValue(Option<String>),
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
            Self::SelectedValue(v) => Self::SelectedValue(v.clone()),
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
            Self::SelectedValue(value) => value.fmt(f),
            Self::Bool(value) => value.fmt(f),
            Self::I32(value) => value.fmt(f),
            Self::U32(value) => value.fmt(f),
            Self::F64(value) => value.fmt(f),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AttributeValueList(Vec<AttributeValue>);

impl AttributeValueList {
    pub fn store_listener(&mut self, index: usize, listener: Box<dyn crate::events::Listener>) {
        if index < self.0.len() {
            self.0[index] = AttributeValue::EventListener(Some(listener));
        } else {
            self.0.push(AttributeValue::EventListener(Some(listener)));
        }
    }

    pub fn check_bool_attribute(&mut self, index: usize, value: bool) -> bool {
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

    pub fn check_i32_attribute(&mut self, index: usize, value: i32) -> bool {
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

    pub fn check_u32_attribute(&mut self, index: usize, value: u32) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::U32(value));
                true
            }
            Some(a) => match a {
                AttributeValue::U32(old_value) if value == *old_value => false,
                AttributeValue::U32(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::U32?"),
            },
        }
    }

    pub fn check_f64_attribute(&mut self, index: usize, value: f64) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::F64(value));
                true
            }
            Some(a) => match a {
                AttributeValue::F64(old_value)
                    if (value - *old_value).abs() < std::f64::EPSILON =>
                {
                    false
                }
                AttributeValue::F64(old_value) => {
                    *old_value = value;
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::F64?"),
            },
        }
    }

    pub fn check_str_attribute(&mut self, index: usize, value: &str) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::String(value.to_string()));
                true
            }
            Some(a) => match a {
                AttributeValue::String(old_value) if value == *old_value => false,
                AttributeValue::String(old_value) => {
                    *old_value = value.to_string();
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::String?"),
            },
        }
    }

    pub fn check_optional_str_attribute(&mut self, index: usize, value: Option<&str>) -> bool {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::SelectedValue(
                    value.map(ToString::to_string),
                ));
                true
            }
            Some(a) => match a {
                AttributeValue::SelectedValue(old_value) if value == old_value.as_deref() => false,
                AttributeValue::SelectedValue(old_value) => {
                    *old_value = value.map(ToString::to_string);
                    true
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::SelectedValue?"),
            },
        }
    }

    pub fn check_str_attribute_and_return_old_value(
        &mut self,
        index: usize,
        value: &str,
    ) -> (bool, Option<String>) {
        match self.0.get_mut(index) {
            None => {
                self.0.push(AttributeValue::String(value.to_string()));
                (true, None)
            }
            Some(a) => match a {
                AttributeValue::String(old_value) if value == *old_value => (false, None),
                AttributeValue::String(old_value) => {
                    let mut value = value.to_string();
                    std::mem::swap(&mut value, old_value);
                    (true, Some(value))
                }
                _ => panic!("Spair's internal error, expected an AttributeValue::String"),
            },
        }
    }
}
