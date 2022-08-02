use super::{
    HtmlElementRender, HtmlElementRenderMut,
    MethodsForDeprecatingAttributesAndElementsWithAmbiguousNames,
    MethodsForHtmlAttributesWithAmbiguousNames, MethodsForHtmlAttributesWithPredifinedValues,
};
use crate::component::Component;
use crate::dom::ElementType;
use crate::render::base::{
    BoolAttributeValue, ElementRender, ElementRenderMut, F64AttributeValue, I32AttributeValue,
    MethodsForEvents, StringAttributeValue, U32AttributeValue,
};
use wasm_bindgen::JsCast;

/// Elements that have attribute `value` are `select`, `input`, `option`, and `textarea`.
/// Apart from `select`, other elements have no issues with the `value` attribute. Setting
/// `value` on a `select` element expect the corresponding `option` element inside the
/// `select` element got highlighted as selected. But setting the value attribute before
/// adding the children (`option` elements) of the select will not work. This trait
/// provide methods to work with attribute value to help handle the issue. But this
/// trait alone can not sovle the issue. We also need HtmlElementRender and HtmlNodeListRender.
pub trait MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement<C: Component>:
    Sized + HtmlElementRenderMut<C>
{
    fn value(mut self, value: impl AttributeValue<C>) -> Self {
        value.render(self.html_element_render_mut());
        self
    }

    fn selected_value(mut self, value: impl AttributeValue<C>) -> Self {
        value.render(self.html_element_render_mut());
        self
    }

    fn selected_index(mut self, value: impl AttributeIndex<C>) -> Self {
        value.render(self.html_element_render_mut());
        self
    }
}

pub trait AttributeValue<C: Component> {
    fn render(self, element: &mut HtmlElementRender<C>);
}

pub trait AttributeIndex<C: Component> {
    fn render(self, element: &mut HtmlElementRender<C>);
}

macro_rules! impl_attribute_value_index_trait_for_types {
    ($($TraitName:ident, $SelfType:ty, $method_name:ident)+) => {$(
        impl<C: Component> $TraitName<C> for $SelfType {
            fn render(self, element: &mut HtmlElementRender<C>) {
                element.$method_name(self);
            }
        }
    )+};
}

impl_attribute_value_index_trait_for_types! {
    AttributeValue, &str,           attribute_value_str
    AttributeValue, String,         attribute_value_string
    AttributeValue, &String,        attribute_value_str
    AttributeValue, Option<&str>,   attribute_value_optional_str
    AttributeValue, Option<String>, attribute_value_optional_string
    AttributeIndex, usize,          attribute_selected_index_usize
    AttributeIndex, Option<usize>,  attribute_selected_index_optional_usize
}

pub trait MethodsForSpecialHtmlAttributes<C: Component>:
    Sized + ElementRenderMut<C> + MethodsForHtmlAttributes<C>
{
    /// Only execute `input.set_checked` if the value changed.
    fn checked_if_changed(mut self, value: bool) -> Self {
        if self
            .element_render_mut()
            .need_to_render_attribute(|al, index| al.check_bool_attribute(index, value))
        {
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
        let element = self.element_render().element();
        if element.element_type() == ElementType::Input {
            let input = element
                .ws_element()
                .unchecked_ref::<web_sys::HtmlInputElement>();
            input.set_checked(value);
        } else {
            log::warn!(".checked() is called on an element that is not <input>");
        }
        self
    }

    fn class(mut self, class_name: &str) -> Self {
        self.element_render_mut().class(class_name);
        self
    }

    fn class_if(mut self, class_on: bool, class_name: &str) -> Self {
        self.element_render_mut().class_if(class_on, class_name);
        self
    }

    /// Set the `first_class` if `first` is true, otherwise, set the `second_class`
    fn class_or(mut self, first: bool, first_class: &str, second_class: &str) -> Self {
        self.element_render_mut()
            .class_or(first, first_class, second_class);
        self
    }

    fn enabled(self, value: bool) -> Self {
        self.disabled(!value)
    }

    fn focus(mut self, value: bool) -> Self {
        self.element_render_mut().focus(value);
        self
    }

    /// This method only accepts a &Route. If you want set `href` with a str, please use `href_str()`.
    /// It is possible to make this method accept both a Route and a str, but I intentionally make
    /// them two separate methods. The purpose is to remind users to use a Route when it's possible.
    fn href(mut self, route: &C::Routes) -> Self {
        self.element_render_mut().href(route);
        self
    }

    fn id(mut self, id: &str) -> Self {
        self.element_render_mut().id(id);
        self
    }

    fn scroll_to_top_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_render()
                .element()
                .scroll_to_view_with_bool(true);
        }
        self
    }

    fn scroll_to_bottom_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_render()
                .element()
                .scroll_to_view_with_bool(false);
        }
        self
    }
}

#[cfg(test)]
use crate::render::html::AllAttributes;
#[cfg(test)]
use crate::render::html::AllElements;

make_trait_for_attribute_methods! {
    TestStructs: (AllElements AllAttributes)
    TraitName: MethodsForHtmlAttributes
    attributes:
        i32     tab_index "tabindex"

        str     abbr
        str     accept
        str     accept_charset "accept-charset"
        str     action
        str     allow
        str     allow_full_screen "allowfullscreen"
        bool    allow_payment_request "allowpaymentrequest"
        str     alt
        bool    auto_play "autoplay"
        str     cite
        //str     class
        u32     cols
        u32     col_span "colspan"
        bool    content_editable "contenteditable"
        bool    controls
        str     coords
        str     data
        str     date_time "datetime"
        bool    default
        str     dir_name "dirname"
        bool    disabled
        str     download
        str     r#for "for"
        str     form
        str     form_action "formaction"
        bool    form_no_validate "formnovalidate"
        str     headers
        u32     height
        bool    hidden
        f64     high
        str     href_str "href" // method named `href` is used for routing
        str     href_lang "hreflang"
        bool    is_map "ismap"
        str     label
        bool    r#loop "loop"
        f64     low
        // ??   max: what type? split into multiple methods?
        i32     max_length "maxlength"
        str     media
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
        bool    read_only "readonly"
        str     rel
        // ??     rellist
        bool    required
        bool    reversed
        u32     rows
        u32     row_span "rowspan"
        // ?? sandbox
        bool    selected
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
        str     title
        str     use_map "usemap"
        u32     width
}

pub struct AttributesOnly<'er, C: Component>(HtmlElementRender<'er, C>);
pub struct StaticAttributesOnly<'er, C: Component>(HtmlElementRender<'er, C>);
pub struct StaticAttributes<'er, C: Component>(HtmlElementRender<'er, C>);

impl<'er, C: Component> AttributesOnly<'er, C> {
    pub(super) fn new(er: HtmlElementRender<'er, C>) -> Self {
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementRender<'er, C> {
        self.0
    }

    pub fn static_attributes_only(self) -> StaticAttributesOnly<'er, C> {
        StaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> StaticAttributesOnly<'er, C> {
    pub(super) fn new(er: HtmlElementRender<'er, C>) -> Self {
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementRender<'er, C> {
        self.0
    }
}

impl<'er, C: Component> StaticAttributes<'er, C> {
    pub(super) fn new(er: HtmlElementRender<'er, C>) -> Self {
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementRender<'er, C> {
        self.0
    }
    pub fn static_attributes_only(self) -> StaticAttributesOnly<'er, C> {
        StaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> ElementRenderMut<C> for AttributesOnly<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        self.0.element_render()
    }
    fn element_render_mut(&mut self) -> &mut ElementRender<C> {
        self.0.element_render_mut()
    }
}
impl<'er, C: Component> HtmlElementRenderMut<C> for AttributesOnly<'er, C> {
    fn html_element_render_mut(&mut self) -> &'er mut HtmlElementRender<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementRenderMut<C> for StaticAttributesOnly<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        self.0.element_render()
    }
    fn element_render_mut(&mut self) -> &mut ElementRender<C> {
        self.0.element_render_mut()
    }
}
impl<'er, C: Component> HtmlElementRenderMut<C> for StaticAttributesOnly<'er, C> {
    fn html_element_render_mut(&mut self) -> &'er mut HtmlElementRender<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementRenderMut<C> for StaticAttributes<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        self.0.element_render()
    }
    fn element_render_mut(&mut self) -> &mut ElementRender<C> {
        self.0.element_render_mut()
    }
}
impl<'er, C: Component> HtmlElementRenderMut<C> for StaticAttributes<'er, C> {
    fn html_element_render_mut(&mut self) -> &'er mut HtmlElementRender<C> {
        &mut self.0
    }
}

impl<'er, C: Component> MethodsForDeprecatingAttributesAndElementsWithAmbiguousNames
    for StaticAttributes<'er, C>
{
}
impl<'er, C: Component> MethodsForHtmlAttributes<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement<C>
    for StaticAttributes<'er, C>
{
}
impl<'er, C: Component> MethodsForSpecialHtmlAttributes<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributesWithPredifinedValues<C>
    for StaticAttributes<'er, C>
{
}

impl<'er, C: Component> MethodsForHtmlAttributes<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement<C>
    for AttributesOnly<'er, C>
{
}
impl<'er, C: Component> MethodsForSpecialHtmlAttributes<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributesWithAmbiguousNames<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributesWithPredifinedValues<C> for AttributesOnly<'er, C> {}

impl<'er, C: Component> MethodsForHtmlAttributes<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement<C>
    for StaticAttributesOnly<'er, C>
{
}
impl<'er, C: Component> MethodsForSpecialHtmlAttributes<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributesWithAmbiguousNames<C>
    for StaticAttributesOnly<'er, C>
{
}
impl<'er, C: Component> MethodsForHtmlAttributesWithPredifinedValues<C>
    for StaticAttributesOnly<'er, C>
{
}

impl<'er, C: Component> MethodsForEvents<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> MethodsForEvents<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForEvents<C> for AttributesOnly<'er, C> {}
