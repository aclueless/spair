use wasm_bindgen::{JsCast, UnwrapThrowExt};

enum Attribute {
    EventListener(Box<dyn crate::events::Listener>),
    String(String),
    Bool(bool),
    I32(i32),
    U32(u32),
    F64(f64),
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

#[derive(Default, Debug)]
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

    fn check_i32_attribute(&mut self, index: usize, value: i32) -> bool {
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

    fn check_u32_attribute(&mut self, index: usize, value: u32) -> bool {
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

    fn check_f64_attribute(&mut self, index: usize, value: f64) -> bool {
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

pub struct StaticAttributes<'a, C: crate::component::Component>(super::ElementUpdater<'a, C>);

impl<'a, C: crate::component::Component> StaticAttributes<'a, C> {
    pub(super) fn new(eu: super::ElementUpdater<'a, C>) -> Self {
        Self(eu)
    }

    pub fn attributes(self) -> Attributes<'a, C> {
        Attributes(self.0)
    }

    pub fn static_nodes(self) -> super::StaticNodesOwned<'a, C> {
        super::StaticNodesOwned::from_el_updater(self.0)
    }

    pub fn nodes(self) -> super::NodesOwned<'a, C> {
        super::NodesOwned::from_el_updater(self.0)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.0.list(items, mode)
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, tag, render, mode)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        self.0.keyed_list(items, mode)
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.0.component(child);
    }
}

pub struct Attributes<'a, C: crate::component::Component>(super::ElementUpdater<'a, C>);

impl<'a, C: crate::component::Component> Attributes<'a, C> {
    pub(super) fn new(eu: super::ElementUpdater<'a, C>) -> Self {
        Self(eu)
    }

    pub fn static_nodes(self) -> super::StaticNodesOwned<'a, C> {
        super::StaticNodesOwned::from_el_updater(self.0)
    }

    pub fn nodes(self) -> super::NodesOwned<'a, C> {
        super::NodesOwned::from_el_updater(self.0)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        I: crate::renderable::ListItem<C>,
    {
        self.0.list(items, mode)
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, tag, render, mode)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(self, items: impl IntoIterator<Item = I>, mode: super::ListElementCreation)
    where
        for<'k> I: super::KeyedListItem<'k, C>,
    {
        self.0.keyed_list(items, mode)
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.0.component(child);
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
        fn $method_name(mut self, value: impl super::AsStr) -> Self {
            self.set_str_attribute($attribute_name, value.as_str());
            self
        }
    };
    (@create $method_name:ident $attribute_name:expr => $attribute_type:ty => $shared_method_name:ident) => {
        fn $method_name(mut self, value: $attribute_type) -> Self {
            self.$shared_method_name($attribute_name, value);
            self
        }
    };
}

mod sealed {
    pub trait AttributeSetter {
        fn ws_html_element(&self) -> &web_sys::HtmlElement;
        fn element_type(&self) -> crate::dom::ElementType;
        fn require_set_listener(&mut self) -> bool;
        fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>);

        // Check if the attribute need to be set (and store the new value for the next check)
        fn check_bool_attribute(&mut self, value: bool) -> bool;
        fn check_str_attribute(&mut self, value: &str) -> bool;
        fn check_i32_attribute(&mut self, value: i32) -> bool;
        fn check_u32_attribute(&mut self, value: u32) -> bool;
        fn check_f64_attribute(&mut self, value: f64) -> bool;

        fn set_selected_value(&mut self, value: Option<&str>);
        fn set_selected_index(&mut self, index: Option<usize>);
    }
}

pub trait AttributeSetter<C>: Sized + sealed::AttributeSetter
where
    C: crate::component::Component,
{
    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    fn done(self) {}

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
    }

    create_methods_for_attributes! {
        str     abbr
        str     accept
        str     accept_charset "accept-charset"
        str     action
        str     allow
        str     allow_full_screen "allowfullscreen"
        bool    allow_payment_request "allowpaymentrequest"
        str     alt
        AsStr   auto_complete "autocomplete"
        bool    auto_play "autoplay"
        str     cite
        //str     class
        u32     cols
        u32     col_span "colspan"
        bool    controls
        str     coords
        AsStr   cross_origin "crossorigin"
        str     data
        str     date_time "datetime"
        AsStr   decoding
        bool    default
        str     dir_name "dirname"
        bool    disabled
        str     download
        AsStr   enc_type "enctype"
        str     r#for "for"
        str     form
        str     form_action "formaction"
        AsStr   form_enc_type "formenctype"
        AsStr   form_method "formmethod"
        bool    form_no_validate "formnovalidate"
        AsStr   form_target "formtarget"
        str     headers
        u32     height
        bool    hidden
        f64     high
        str     href_str "href" // method named `href` is used for routing
        str     href_lang "hreflang"
        bool    is_map "ismap"
        AsStr   kind
        str     label
        bool    r#loop "loop"
        f64     low
        // ??   max: what type? split into multiple methods?
        i32     max_length "maxlength"
        str     media
        AsStr   method
        // ??   min: similar to max
        i32     min_length "minlength"
        bool    multiple
        bool    muted
        str     name
        bool    no_validate "novalidate"
        bool    open
        f64     optimum
        str     pattern
        str     ping
        str     placeholder
        str     poster
        bool    plays_inline "playsinline"
        AsStr   pre_load "preload"
        bool    read_only "readonly"
        AsStr   referrer_policy "referrerpolicy"
        str     rel
        // ??     rellist
        bool    required
        bool    reversed
        u32     rows
        u32     row_span "rowspan"
        // ?? sandbox
        bool    selected
        AsStr   scope
        u32     size
        str     sizes
        u32     span
        str     src
        str     src_doc "srcdoc"
        str     src_lang "srclang"
        str     src_set "srcset"
        i32     start
        str     step
        str     style
        AsStr   target
        str     title
        AsStr   r#type "type"
        str     use_map "usemap"
        u32     width
        AsStr   wrap
    }

    /// Only execute `input.set_checked` if the value changed.
    fn checked_if_changed(mut self, value: bool) -> Self {
        if self.check_bool_attribute(value) {
            self.checked(value)
        } else {
            self
        }
    }

    /// Always execute `input.set_checked` with the given value. This is
    /// useful in situation like in TodoMVC example. TodoMVC spec requires
    /// that when the app in a filtered mode, for example, just display
    /// active todos, if an item is checked (completed) by clicking the
    /// input, the app should hide the todo item. In such a situation, the
    /// DOM item is checked, but Spair DOM is not checked yet. But the
    /// checked item was filtered out (hidden), and only active todos
    /// are displayed, all of them are unchecked which match the state in
    /// Spair DOM, hence Spair skip setting check, leaving the DOM checked
    /// but display an unchecked item. In my understand, this only occurs
    /// with non-keyed list. I choose always setting checked to avoid
    /// surprise for new users. `checked_if_changed` can be used to reduce
    /// interaction with DOM if it does not bug you.
    fn checked(self, value: bool) -> Self {
        let element = self.ws_element();
        if self.element_type() == super::ElementType::Input {
            let input = element.unchecked_ref::<web_sys::HtmlInputElement>();
            input.set_checked(value);
        } else {
            log::warn!(".checked() is called on an element that is not <input>");
        }
        self
    }

    /// This method should only execute in static mode.
    fn class(self, class_name: &str) -> Self {
        self.ws_element()
            .class_list()
            .add_1(class_name)
            .expect_throw("Unable to add class");
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

    fn enabled(self, value: bool) -> Self {
        self.disabled(!value)
    }

    fn focus(mut self, value: bool) -> Self {
        if value && self.check_bool_attribute(value) {
            self.ws_html_element()
                .focus()
                .expect_throw("Unable to set focus");
        }
        self
    }

    fn href(mut self, value: &C::Routes) -> Self {
        use crate::routing::Routes;
        let url = value.url();
        if self.check_str_attribute(&url) {
            self.set_str_attribute("href", &url);
        }
        self
    }

    fn id(mut self, id: &str) -> Self {
        if self.check_str_attribute(id) {
            self.ws_element().set_id(id);
        }
        self
    }

    fn selected_value(mut self, value: Option<&str>) -> Self {
        if self.element_type() == super::ElementType::Select {
            // TODO: check to find change of value?

            // It has no effect if you set a value for
            // a <select> element before adding its <option>s,
            // the hacking should finish in the list() method.
            // Is there a better solution?
            self.set_selected_value(value);
        } else {
            log::warn!(".selected_value() can only be called on <select>");
        }
        self
    }

    fn selected_index(mut self, index: Option<usize>) -> Self {
        if self.element_type() == super::ElementType::Select {
            // TODO: check to find change of index?

            // It has no effect if you set a selected index for
            // a <select> element before adding its <option>s,
            // the hacking should finish in the list() method.
            // Is there a better solution?
            self.set_selected_index(index);
        } else {
            log::warn!(".selected_index() is called on an element that is not <select>");
        }
        self
    }

    fn value(mut self, value: &str) -> Self {
        if self.check_str_attribute(value) {
            let element = self.ws_element();
            match self.element_type() {
                super::ElementType::Input => {
                    let input = element.unchecked_ref::<web_sys::HtmlInputElement>();
                    input.set_value(value);
                }
                super::ElementType::Select => {
                    // It has no effect if you set a value for
                    // a <select> element before adding its <option>s,
                    // the hacking should finish in the list() method.
                    // Is there a better solution?
                    self.set_selected_value(Some(value));
                }
                super::ElementType::TextArea => {
                    let text_area = element.unchecked_ref::<web_sys::HtmlTextAreaElement>();
                    text_area.set_value(value);
                }
                super::ElementType::Option => {
                    let option = element.unchecked_ref::<web_sys::HtmlOptionElement>();
                    option.set_value(value);
                }
                super::ElementType::Other => {
                    log::warn!(
                        ".value() is called on an element that is not <input>, <select>, <option>, <textarea>"
                    );
                }
            }
        }
        self
    }
}

impl<'a, C: crate::component::Component> AttributeSetter<C> for super::StaticAttributes<'a, C>
where
    C: crate::component::Component,
{
    fn ws_element(&self) -> &web_sys::Element {
        &self.0.element.ws_element
    }
}

impl<'a, C: crate::component::Component> sealed::AttributeSetter
    for super::StaticAttributes<'a, C>
{
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.element.ws_element.unchecked_ref()
    }

    fn element_type(&self) -> super::ElementType {
        self.0.element.element_type
    }
    fn require_set_listener(&mut self) -> bool {
        if self.0.status == super::ElementStatus::Existing {
            // When self.require_init == false, self.store_listener will not be invoked.
            // We must update the index here to count over the static events.
            self.0.index += 1;
            false
        } else {
            // A cloned element requires its event handlers to be set because the events
            // are not cloned.
            true
        }
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0
            .element
            .attributes
            .store_listener(self.0.index, listener);
        self.0.index += 1;
    }

    fn check_bool_attribute(&mut self, _value: bool) -> bool {
        self.0.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_str_attribute(&mut self, _value: &str) -> bool {
        self.0.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_i32_attribute(&mut self, _value: i32) -> bool {
        self.0.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_u32_attribute(&mut self, _value: u32) -> bool {
        self.0.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_f64_attribute(&mut self, _value: f64) -> bool {
        self.0.status == super::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn set_selected_value(&mut self, value: Option<&str>) {
        self.0.select_element_value.set_selected_value(value);
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.0.select_element_value.set_selected_index(index);
    }
}

impl<'a, C: crate::component::Component> AttributeSetter<C> for super::Attributes<'a, C>
where
    C: crate::component::Component,
{
    fn ws_element(&self) -> &web_sys::Element {
        &self.0.element.ws_element
    }
}

impl<'a, C: crate::component::Component> sealed::AttributeSetter for super::Attributes<'a, C> {
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.element.ws_element.unchecked_ref()
    }

    fn element_type(&self) -> super::ElementType {
        self.0.element.element_type
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0
            .element
            .attributes
            .store_listener(self.0.index, listener);
        self.0.index += 1;
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_bool_attribute(self.0.index, value);
        self.0.index += 1;
        rs
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_str_attribute(self.0.index, value);
        self.0.index += 1;
        rs
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_i32_attribute(self.0.index, value);
        self.0.index += 1;
        rs
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_u32_attribute(self.0.index, value);
        self.0.index += 1;
        rs
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        let rs = self
            .0
            .element
            .attributes
            .check_f64_attribute(self.0.index, value);
        self.0.index += 1;
        rs
    }

    fn set_selected_value(&mut self, value: Option<&str>) {
        self.0.select_element_value.set_selected_value(value);
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.0.select_element_value.set_selected_index(index);
    }
}
