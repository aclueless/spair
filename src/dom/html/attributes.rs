use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub trait AttributeValueSetter: Sized + crate::dom::attributes::AttributeSetter {
    fn set_selected_value(&mut self, value: Option<&str>);
    fn set_selected_index(&mut self, index: Option<usize>);

    fn value_str(&mut self, value: &str) {
        if self.check_str_attribute(value) {
            let element = self.ws_element();
            match self.element_type() {
                crate::dom::ElementType::Input => {
                    let input = element.unchecked_ref::<web_sys::HtmlInputElement>();
                    input.set_value(value);
                }
                crate::dom::ElementType::Select => {
                    // It has no effect if you set a value for
                    // a <select> element before adding its <option>s,
                    // the hacking should finish in the list() method.
                    // Is there a better solution?
                    self.set_selected_value(Some(value));
                }
                crate::dom::ElementType::TextArea => {
                    let text_area = element.unchecked_ref::<web_sys::HtmlTextAreaElement>();
                    text_area.set_value(value);
                }
                crate::dom::ElementType::Option => {
                    let option = element.unchecked_ref::<web_sys::HtmlOptionElement>();
                    option.set_value(value);
                }
                crate::dom::ElementType::Other => {
                    log::warn!(
                        ".value() is called on an element that is not <input>, <select>, <option>, <textarea>"
                    );
                }
            }
        }
    }
}

pub trait AttributeSetter<C>: Sized + AttributeValueSetter
where
    C: crate::component::Component,
{
    fn bool_attr(mut self, name: &str, value: bool) -> Self {
        self.set_bool_attribute(name, value);
        self
    }

    fn str_attr(mut self, name: &str, value: &str) -> Self {
        self.set_str_attribute(name, value);
        self
    }

    fn i32_attr(mut self, name: &str, value: i32) -> Self {
        self.set_i32_attribute(name, value);
        self
    }

    fn u32_attr(mut self, name: &str, value: u32) -> Self {
        self.set_u32_attribute(name, value);
        self
    }

    fn f64_attr(mut self, name: &str, value: f64) -> Self {
        self.set_f64_attribute(name, value);
        self
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
        bool    content_editable "contenteditable"
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
        u32     span_attr "span" // rename to `span_attr` to avoid conflict with DomBuilder::span
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
        if self.element_type() == crate::dom::ElementType::Input {
            let input = element.unchecked_ref::<web_sys::HtmlInputElement>();
            input.set_checked(value);
        } else {
            log::warn!(".checked() is called on an element that is not <input>");
        }
        self
    }

    fn class(mut self, class_name: &str) -> Self {
        let (changed, old_value) = self.check_str_attribute_and_return_old_value(class_name);
        if let Some(old_value) = old_value {
            self.ws_element()
                .class_list()
                .remove_1(&old_value)
                .expect_throw("Unable to remove old class");
        }
        if changed {
            self.ws_element()
                .class_list()
                .add_1(class_name)
                .expect_throw("Unable to add new class");
        }
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

    fn max(self, value: impl AttributeMax<Self>) -> Self {
        value.update(self)
    }

    fn min(self, value: impl AttributeMin<Self>) -> Self {
        value.update(self)
    }

    // selected_value and value should be integrated in to one method?
    fn selected_value(mut self, value: Option<&str>) -> Self {
        if self.element_type() == crate::dom::ElementType::Select {
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
        if self.element_type() == crate::dom::ElementType::Select {
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

    fn value(self, value: impl AttributeValue<Self>) -> Self {
        value.update(self)
    }
}

pub struct StaticAttributes<'a, C>(crate::dom::HtmlUpdater<'a, C>);

impl<'a, C> From<crate::dom::HtmlUpdater<'a, C>> for StaticAttributes<'a, C> {
    fn from(u: crate::dom::HtmlUpdater<'a, C>) -> Self {
        Self(u)
    }
}

impl<'a, C: crate::component::Component> StaticAttributes<'a, C> {
    /// Use this method when you are done with your object. It is useful in single-line closures
    /// where you don't want to add a semicolon `;` but the compiler complains that "expected `()`
    /// but found `something-else`"
    pub fn done(self) {}

    pub fn nodes(self) -> crate::dom::NodesOwned<'a, C> {
        self.0.nodes()
    }

    pub fn static_nodes(self) -> crate::dom::StaticNodesOwned<'a, C> {
        self.0.static_nodes()
    }

    #[cfg(feature = "svg")]
    pub fn svg(self, f: impl FnOnce(crate::dom::SvgUpdater<C>)) -> crate::dom::NodesOwned<'a, C> {
        // Although this is StaticAttributes. But we are switching from setting attributes
        // to adding nodes. The default mode for adding nodes is update-mode.
        self.0.svg(f)
    }

    pub fn render(self, value: impl crate::dom::Render<C>) -> crate::dom::NodesOwned<'a, C> {
        self.0.render(value)
    }

    pub fn render_ref(
        self,
        value: &impl crate::dom::RenderRef<C>,
    ) -> crate::dom::NodesOwned<'a, C> {
        self.0.render_ref(value)
    }

    pub fn r#static(
        self,
        value: impl crate::dom::StaticRender<C>,
    ) -> crate::dom::NodesOwned<'a, C> {
        self.0.r#static(value)
    }

    pub fn list<I>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
    ) -> crate::dom::node_list_extensions::NodeListExtensions<'a>
    where
        I: Copy,
        I: crate::dom::ListItemRender<C>,
    {
        self.0
            .list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
        tag: &'a str,
        render: R,
    ) -> crate::dom::node_list_extensions::NodeListExtensions<'a>
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.0.list_with_render(items, mode, tag, render)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
    ) where
        I: Copy,
        for<'k> I: crate::dom::Keyed<'k> + crate::dom::ListItemRender<C>,
    {
        self.0
            .keyed_list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::key, I::render);
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list_with_render<I, G, K, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
        tag: &'a str,
        get_key: G,
        render: R,
    ) where
        I: Copy,
        G: Fn(I) -> K,
        K: Into<crate::dom::Key> + PartialEq<crate::dom::Key>,
        for<'u> R: Fn(I, super::HtmlUpdater<'u, C>),
    {
        self.0
            .keyed_list_with_render(items, mode, tag, get_key, render);
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.0.component(child);
    }
}

impl<'a, C: crate::component::Component> AttributeValueSetter for StaticAttributes<'a, C> {
    fn set_selected_value(&mut self, value: Option<&str>) {
        self.0.set_selected_value(value);
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.0.set_selected_index(index);
    }
}
impl<'a, C: crate::component::Component> AttributeSetter<C> for StaticAttributes<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> crate::dom::attributes::EventSetter
    for StaticAttributes<'a, C>
where
    C: crate::component::Component,
{
}

impl<'a, C: crate::component::Component> crate::dom::attributes::AttributeSetter
    for StaticAttributes<'a, C>
{
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.ws_html_element()
    }

    fn ws_element(&self) -> &web_sys::Element {
        self.0.ws_element()
    }

    fn element_type(&self) -> crate::dom::ElementType {
        self.0.element_type()
    }

    fn require_set_listener(&mut self) -> bool {
        if self.0.status() == crate::dom::ElementStatus::Existing {
            // self.store_listener will not be invoked.
            // We must update the index here to count over the static events.
            self.0.u.next_index();
            false
        } else {
            // A cloned element requires its event handlers to be set because the events
            // are not cloned.
            true
        }
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0.store_listener(listener)
    }

    fn check_bool_attribute(&mut self, _value: bool) -> bool {
        self.0.status() == crate::dom::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_str_attribute(&mut self, _value: &str) -> bool {
        self.0.status() == crate::dom::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_i32_attribute(&mut self, _value: i32) -> bool {
        self.0.status() == crate::dom::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_u32_attribute(&mut self, _value: u32) -> bool {
        self.0.status() == crate::dom::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_f64_attribute(&mut self, _value: f64) -> bool {
        self.0.status() == crate::dom::ElementStatus::JustCreated
        // no need to store the value for static attributes
    }

    fn check_str_attribute_and_return_old_value(&mut self, _value: &str) -> (bool, Option<String>) {
        (
            self.0.status() == crate::dom::ElementStatus::JustCreated,
            None,
        )
    }
}

// TODO: Should all these (below) be produced by macros?
pub trait AttributeValue<U> {
    fn update(self, u: U) -> U;
}

// &str
impl<'a, C: crate::component::Component> AttributeValue<crate::dom::HtmlUpdater<'a, C>> for &str {
    fn update(self, mut u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.value_str(self);
        u
    }
}

impl<'a, C: crate::component::Component> AttributeValue<crate::dom::StaticAttributes<'a, C>>
    for &str
{
    fn update(
        self,
        mut u: crate::dom::StaticAttributes<'a, C>,
    ) -> crate::dom::StaticAttributes<'a, C> {
        u.value_str(self);
        u
    }
}

// &String
impl<'a, C: crate::component::Component> AttributeValue<crate::dom::HtmlUpdater<'a, C>>
    for &String
{
    fn update(self, mut u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.value_str(self);
        u
    }
}

impl<'a, C: crate::component::Component> AttributeValue<crate::dom::StaticAttributes<'a, C>>
    for &String
{
    fn update(
        self,
        mut u: crate::dom::StaticAttributes<'a, C>,
    ) -> crate::dom::StaticAttributes<'a, C> {
        u.value_str(self);
        u
    }
}

// Option<&str>
impl<'a, C: crate::component::Component> AttributeValue<crate::dom::HtmlUpdater<'a, C>>
    for Option<&str>
{
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.selected_value(self)
    }
}

impl<'a, C: crate::component::Component> AttributeValue<crate::dom::StaticAttributes<'a, C>>
    for Option<&str>
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.selected_value(self)
    }
}

// f64
impl<'a, C: crate::component::Component> AttributeValue<crate::dom::HtmlUpdater<'a, C>> for f64 {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.f64_attr("value", self)
    }
}

impl<'a, C: crate::component::Component> AttributeValue<crate::dom::StaticAttributes<'a, C>>
    for f64
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.f64_attr("value", self)
    }
}

pub trait AttributeMax<U> {
    fn update(self, u: U) -> U;
}

// &str
impl<'a, C: crate::component::Component> AttributeMax<crate::dom::HtmlUpdater<'a, C>> for &str {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.str_attr("max", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMax<crate::dom::StaticAttributes<'a, C>>
    for &str
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.str_attr("max", self)
    }
}

// &String
impl<'a, C: crate::component::Component> AttributeMax<crate::dom::HtmlUpdater<'a, C>> for &String {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.str_attr("max", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMax<crate::dom::StaticAttributes<'a, C>>
    for &String
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.str_attr("max", self)
    }
}

// f64
impl<'a, C: crate::component::Component> AttributeMax<crate::dom::HtmlUpdater<'a, C>> for f64 {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.f64_attr("max", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMax<crate::dom::StaticAttributes<'a, C>> for f64 {
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.f64_attr("max", self)
    }
}

pub trait AttributeMin<U> {
    fn update(self, u: U) -> U;
}

// &str
impl<'a, C: crate::component::Component> AttributeMin<crate::dom::HtmlUpdater<'a, C>> for &str {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.str_attr("min", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMin<crate::dom::StaticAttributes<'a, C>>
    for &str
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.str_attr("min", self)
    }
}

// &String
impl<'a, C: crate::component::Component> AttributeMin<crate::dom::HtmlUpdater<'a, C>> for &String {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.str_attr("min", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMin<crate::dom::StaticAttributes<'a, C>>
    for &String
{
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.str_attr("min", self)
    }
}

// f64
impl<'a, C: crate::component::Component> AttributeMin<crate::dom::HtmlUpdater<'a, C>> for f64 {
    fn update(self, u: crate::dom::HtmlUpdater<'a, C>) -> crate::dom::HtmlUpdater<'a, C> {
        u.f64_attr("min", self)
    }
}

impl<'a, C: crate::component::Component> AttributeMin<crate::dom::StaticAttributes<'a, C>> for f64 {
    fn update(self, u: crate::dom::StaticAttributes<'a, C>) -> crate::dom::StaticAttributes<'a, C> {
        u.f64_attr("min", self)
    }
}
