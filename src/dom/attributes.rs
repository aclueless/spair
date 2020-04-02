use wasm_bindgen::{JsCast, UnwrapThrowExt};

enum Attribute {
    EventListener(Box<dyn crate::events::Listener>),
    Bool(bool),
    String(String),
}

#[derive(Default)]
pub struct AttributeList(Vec<Attribute>);

impl AttributeList {
    fn store_listener(&mut self, index: usize, listener: Box<dyn crate::events::Listener>) {
        if index < self.0.len() {
            self.0[index] = Attribute::EventListener(listener);
        } else {
            self.0.push(Attribute::EventListener(listener));
        }
    }

    fn check_bool_attribute(&mut self, index: usize, value: bool) -> bool {
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

    fn check_str_attribute(&mut self, index: usize, value: &str) -> bool {
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
}

pub struct StaticAttributes<'a, C>(super::ElementHandle<'a, C>);

impl<'a, C> StaticAttributes<'a, C> {
    pub(super) fn new(handle: super::ElementHandle<'a, C>) -> Self {
        Self(handle)
    }

    pub fn attributes(self) -> Attributes<'a, C> {
        Attributes(self.0)
    }

    pub fn static_nodes(self) -> super::StaticNodes<'a, C> {
        super::StaticNodes::from_handle(self.0)
    }

    pub fn nodes(self) -> super::Nodes<'a, C> {
        super::Nodes::from_handle(self.0)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, state: &C)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.0.list(items, state)
    }
}

pub struct Attributes<'a, C>(super::ElementHandle<'a, C>);

impl<'a, C> Attributes<'a, C> {
    pub(super) fn new(handle: super::ElementHandle<'a, C>) -> Self {
        Self(handle)
    }

    pub fn static_nodes(self) -> super::StaticNodes<'a, C> {
        super::StaticNodes::from_handle(self.0)
    }

    pub fn nodes(self) -> super::Nodes<'a, C> {
        super::Nodes::from_handle(self.0)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, state: &C)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.0.list(items, state)
    }
}

macro_rules! create_methods_for_events {
    ($($method_name:ident $EventType:ident,)+) => {
        $(
            fn $method_name<F>(mut self, f: F) -> Self
            where F: crate::events::$EventType
            {
                if self.require_set_listener() {
                    let listener = crate::events::$EventType::on(f, self.ws_element().as_ref());
                    self.store_listener(listener);
                }
                self
            }
        )+
    }
}

macro_rules! create_methods_for_attributes {
    ($type:ty => $method_name:ident [$($attribute:ident)+]) => {
        $(
            fn $attribute(mut self, value: $type) -> Self {
                self.$method_name(stringify!($attribute), value);
                self
            }
        )+
    }
}

mod sealed {
    pub trait AttributeSetter {
        fn ws_html_element(&self) -> &web_sys::HtmlElement;
        fn require_set_listener(&mut self) -> bool;
        fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>);

        // Check if the attribute need to be set (and store the new value for the next check)
        fn check_bool_attribute(&mut self, value: bool) -> bool;
        fn check_str_attribute(&mut self, value: &str) -> bool;
    }
}

pub trait AttributeSetter<C>: Sized + sealed::AttributeSetter
where
    C: crate::component::Component,
{
    fn ws_element(&self) -> &web_sys::Element;

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

    fn focus(self, value: bool) -> Self {
        if value {
            self.ws_html_element()
                .focus()
                .expect_throw("Unable to set focus");
        }
        self
    }

    fn value(mut self, value: &str) -> Self {
        if self.check_str_attribute(value) {
            let element = self.ws_element();
            if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
                input.set_value(value);
            } else if let Some(select) = element.dyn_ref::<web_sys::HtmlSelectElement>() {
                select.set_value(value);
            } else if let Some(text_area) = element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                text_area.set_value(value);
            } else {
                log::warn!(
                    ".value is called on an element that is not <input>, <select>, <textarea>"
                );
            }
        }
        self
    }

    fn id(self, id: &str) -> Self {
        self.ws_element().set_id(id);
        self
    }

    fn r#type(mut self, value: impl super::AsStr) -> Self {
        self.set_str_attribute("type", value.as_str());
        self
    }

    fn r#for(mut self, value: &str) -> Self {
        self.set_str_attribute("for", value);
        self
    }

    fn class_if(mut self, class_name: &str, class_on: bool) -> Self {
        if self.check_bool_attribute(class_on) {
            if class_on {
                self.ws_element()
                    .class_list()
                    .add_1(class_name)
                    .expect_throw("Unable to add class");
            } else {
                self.ws_element()
                    .class_list()
                    .remove_1(class_name)
                    .expect_throw("Unable to remove class");
            }
        }
        self
    }

    fn href(mut self, value: C::Routes) -> Self {
        use crate::routing::Routes;
        self.set_str_attribute("href", &value.url());
        self
    }

    fn href_str(mut self, value: &str) -> Self {
        self.set_str_attribute("href", value);
        self
    }

    create_methods_for_events! {
        on_click Click,
        on_double_click DoubleClick,
        on_change Change,
        on_key_press KeyPress,
        on_blur Blur,
        on_focus Focus,
    }

    create_methods_for_attributes! {
        bool => set_bool_attribute [
            checked
            disabled
            hidden
            readonly
        ]
    }
    create_methods_for_attributes! {
        &str => set_str_attribute [
            class
            placeholder
        ]
    }
}

impl<'a, C> AttributeSetter<C> for super::StaticAttributes<'a, C>
where
    C: crate::component::Component,
{
    fn ws_element(&self) -> &web_sys::Element {
        &self.0.element.ws_element
    }
}

impl<'a, C> sealed::AttributeSetter for super::StaticAttributes<'a, C> {
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.element.ws_element.unchecked_ref()
    }

    fn require_set_listener(&mut self) -> bool {
        if self.0.extra.status == super::ElementStatus::Existing {
            // When self.require_init == false, self.store_listener will not be invoked.
            // We must update the index here to count over the static events.
            self.0.extra.index += 1;
            false
        } else {
            // A cloned element requires its event handlers to be set because they may rely
            // on their index or id...
            true
        }
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0
            .element
            .attributes
            .store_listener(self.0.extra.index, listener);
        self.0.extra.index += 1;
    }

    fn check_bool_attribute(&mut self, _value: bool) -> bool {
        self.0.extra.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_str_attribute(&mut self, _value: &str) -> bool {
        self.0.extra.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }
}

impl<'a, C> AttributeSetter<C> for super::Attributes<'a, C>
where
    C: crate::component::Component,
{
    fn ws_element(&self) -> &web_sys::Element {
        &self.0.element.ws_element
    }
}

impl<'a, C> sealed::AttributeSetter for super::Attributes<'a, C> {
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.element.ws_element.unchecked_ref()
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0
            .element
            .attributes
            .store_listener(self.0.extra.index, listener);
        self.0.extra.index += 1;
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_bool_attribute(self.0.extra.index, value);
        self.0.extra.index += 1;
        rs
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_str_attribute(self.0.extra.index, value);
        self.0.extra.index += 1;
        rs
    }
}
