use super::{HtmlElementUpdater, HtmlElementUpdaterMut};
use crate::{
    component::Component,
    dom::AttributeValueList,
    render::base::{
        AttributeMinMax, BoolAttributeValue, Class, ElementUpdater, ElementUpdaterMut,
        F64AttributeValue, I32AttributeValue, MethodsForEvents, StringAttributeValue,
        U32AttributeValue,
    },
};

#[cfg(feature = "queue-render")]
use crate::{
    dom::WsElement,
    queue_render::value::{MapValue, QrVal},
};

macro_rules! make_traits_for_property_values {
    (
        $UpdaterType:ident
        $(
            $TraitName:ident {
                $(
                    $attribute_type:ty,
                    $method_name:ident
                    $ws_method_for_qr:ident $qr_method_name:ident $qrm_method_name:ident,
                )+
            }
        )+
    ) => {
        $(
            pub trait $TraitName<C: Component> {
                fn render(self, element: &mut $UpdaterType<C>);
            }
            $(
                impl<C: Component> $TraitName<C> for $attribute_type {
                    fn render(self, element: &mut $UpdaterType<C>) {
                        element.$method_name(self);
                    }
                }
                make_traits_for_property_values! {
                    @each_queue_render
                    $UpdaterType
                    $TraitName
                    $attribute_type,
                    $ws_method_for_qr $qr_method_name $qrm_method_name
                }
            )+
        )+
    };
    (
        @each_queue_render
        $UpdaterType:ident
        $TraitName:ident
        $attribute_type:ty,
        NO_QUEUE_RENDER NO_QUEUE_RENDER NO_QUEUE_RENDER
    ) => {
    };
    (
        @each_queue_render
        $UpdaterType:ident
        $TraitName:ident
        $attribute_type:ty,
        $ws_method_for_qr:ident $qr_method_name:ident $qrm_method_name:ident
    ) => {
        #[cfg(feature = "queue-render")]
        impl<C: Component> $TraitName<C> for &QrVal<$attribute_type> {
            fn render(self, element: &mut $UpdaterType<C>) {
                element.$qr_method_name(WsElement::$ws_method_for_qr, self);
            }
        }
        #[cfg(feature = "queue-render")]
        impl<C: Component, T: 'static> $TraitName<C> for MapValue<C, T, $attribute_type> {
            fn render(self, element: &mut $UpdaterType<C>) {
                element.$qrm_method_name(WsElement::$ws_method_for_qr, self);
            }
        }
    };
}

make_traits_for_property_values! {
    HtmlElementUpdater
    PropertyValue {
        &str,           selected_value_str              NO_QUEUE_RENDER NO_QUEUE_RENDER NO_QUEUE_RENDER,
        Option<&str>,   selected_value_optional_str     NO_QUEUE_RENDER NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String,         selected_value_string           set_value_for_qr qr_property qrm_property,
        &String,        selected_value_str              NO_QUEUE_RENDER NO_QUEUE_RENDER NO_QUEUE_RENDER,
        Option<String>, selected_value_optional_string  set_value_for_qr_optional qr_property qrm_property,
    }
    PropertyIndex {
        usize,          selected_index_usize            set_selected_index_ref qr_property qrm_property,
        Option<usize>,  selected_index_optional_usize   set_selected_index_optional qr_property qrm_property,
    }
}

/// Elements that have attribute/property `value` are `select`, `input`,
/// `option`, and `textarea`. Apart from `select`, other elements have
/// no issues with the `value` property. Setting `value` on a `select`
/// element expect the corresponding `option` element inside the
/// `select` element got highlighted as an selected item. But setting
/// the value property before adding the children (`option` elements)
/// of the select will not work. This trait provide methods to work
/// with attribute value to help handle the issue. But this trait alone
/// can not sovle the issue. We also need HtmlElementUpdater and
/// HtmlNodesUpdater.
pub trait MethodsForSelectedValueSelectedIndex<C: Component>:
    Sized + HtmlElementUpdaterMut<C>
{
    fn value(mut self, value: impl PropertyValue<C>) -> Self {
        value.render(self.html_element_updater_mut());
        self
    }

    fn selected_value(mut self, value: impl PropertyValue<C>) -> Self {
        value.render(self.html_element_updater_mut());
        self
    }

    fn selected_index(mut self, value: impl PropertyIndex<C>) -> Self {
        value.render(self.html_element_updater_mut());
        self
    }
}

make_traits_for_property_values! {
    ElementUpdater
    PropertyChecked {
        bool, checked checked_ref qr_property qrm_property,
    }
    AttributeEnabled {
        bool, enabled enabled_ref qr_property qrm_property,
    }
    ActionFocus {
        bool, focus focus_ref qr_property qrm_property,
    }
}

pub trait HamsHandMade<C: Component>:
    Sized + ElementUpdaterMut<C> + HamsForDistinctNames<C>
{
    fn done(self) {}

    /// Only execute `input.set_checked` if the value changed. But it's safer
    /// to use `.checked()` instead.
    fn checked_if_changed(mut self, value: bool) -> Self {
        if self
            .element_updater_mut()
            .must_render_attribute(value, AttributeValueList::check_bool_attribute)
        {
            self.checked(value)
        } else {
            self
        }
    }

    /// The issue describes here does not affect elements in queue render
    /// list or keyed list.
    ///
    /// In incremental mode, this method always execute `input.set_checked`
    /// with the given value. This is useful in situation like in TodoMVC
    /// example. TodoMVC specification requires that when the app in a
    /// filtered mode, for example, 'active' mode just display
    /// active todo-items, if an item is checked (completed) by clicking the
    /// input, the app should hide the todo item. In such a situation, the
    /// DOM item is checked, but Spair DOM is not checked yet. But the
    /// checked item was filtered out (hidden), and only active todos
    /// are displayed, all of them are unchecked which match the state in
    /// Spair DOM, hence Spair skip setting check, leaving the DOM checked
    /// but display an unchecked item. In my understand, this only occurs
    /// with non-keyed list, keyed_list is not affect by this.
    /// I choose to always set checked to avoid surprises for new users.
    /// `checked_if_changed` can be used to reduce interaction with DOM if
    /// it does not bug you.
    fn checked(mut self, value: impl PropertyChecked<C>) -> Self {
        value.render(self.element_updater_mut());
        self
    }

    fn class(mut self, value: impl Class<C>) -> Self {
        value.render(self.element_updater_mut());
        self
    }

    fn class_if(mut self, class_on: bool, class_name: &str) -> Self {
        self.element_updater_mut().class_if(class_on, class_name);
        self
    }

    /// Set the `first_class` if `first` is true, otherwise, set the `second_class`
    fn class_or(mut self, first: bool, first_class: &str, second_class: &str) -> Self {
        self.element_updater_mut()
            .class_or(first, first_class, second_class);
        self
    }

    fn enabled(mut self, value: impl AttributeEnabled<C>) -> Self {
        value.render(self.element_updater_mut());
        self
    }

    fn focus(mut self, value: impl ActionFocus<C>) -> Self {
        value.render(self.element_updater_mut());
        self
    }

    /// This method only accepts a &Route. If you want set `href` with a str, please use `href_str()`.
    /// It is possible to make this method accept either a Route or a str, but I intentionally make
    /// them two separate methods. The purpose is to remind users to use a Route when it's possible.
    fn href(mut self, route: &C::Routes) -> Self {
        self.element_updater_mut().href(route);
        self
    }

    fn id(mut self, id: &str) -> Self {
        self.element_updater_mut().id(id);
        self
    }

    fn scroll_to_top_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_updater()
                .element()
                .ws_element()
                .scroll_to_view_with_bool(true);
        }
        self
    }

    fn scroll_to_bottom_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_updater()
                .element()
                .ws_element()
                .scroll_to_view_with_bool(false);
        }
        self
    }
}

#[cfg(test)]
use crate::render::html::TestHtmlMethods;

make_trait_for_attribute_methods! {
    TestStructs: (TestHtmlMethods)
    TraitName: HamsForDistinctNames
    attributes:
        i32     tab_index "tabindex"

        // moved to ../attributes_elements_with_ambiguous_names
        // str     abbr

        str     accept
        str     accept_charset "accept-charset"
        str     action
        str     allow
        str     allow_full_screen "allowfullscreen"
        bool    allow_payment_request "allowpaymentrequest"
        str     alt
        bool    auto_play "autoplay"

        // moved to ../attributes_elements_with_ambiguous_names
        // str     cite

        //str     class
        u32     cols
        u32     col_span "colspan"
        bool    content_editable "contenteditable"
        bool    controls
        str     coords

        // moved to ../attributes_elements_with_ambiguous_names
        // str     data

        str     date_time "datetime"
        bool    default
        str     dir_name "dirname"
        bool    disabled
        str     download
        str     r#for "for"

        // moved to ../attributes_elements_with_ambiguous_names
        // str     form

        str     form_action "formaction"
        bool    form_no_validate "formnovalidate"
        str     headers
        u32     height
        bool    hidden
        f64     high
        str     href_str "href" // method named `href` is used for routing
        str     href_lang "hreflang"
        bool    is_map "ismap"

        // moved to ../attributes_elements_with_ambiguous_names
        // str     label

        bool    r#loop "loop"
        f64     low
        AttributeMinMax  max
        i32     max_length "maxlength"
        str     media
        AttributeMinMax  min
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

        // moved to ../attributes_elements_with_ambiguous_names
        // u32     span

        str     src
        str     src_doc "srcdoc"
        str     src_lang "srclang"
        str     src_set "srcset"
        i32     start
        f64     step
        str     style
        str     title
        str     use_map "usemap"
        u32     width
}

pub struct AttributesOnly<'er, C: Component>(HtmlElementUpdater<'er, C>);
pub struct StaticAttributesOnly<'er, C: Component>(HtmlElementUpdater<'er, C>);
pub struct StaticAttributes<'er, C: Component>(HtmlElementUpdater<'er, C>);

impl<'er, C: Component> AttributesOnly<'er, C> {
    pub(super) fn new(er: HtmlElementUpdater<'er, C>) -> Self {
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementUpdater<'er, C> {
        self.0
    }

    pub fn static_attributes_only(self) -> StaticAttributesOnly<'er, C> {
        StaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> StaticAttributesOnly<'er, C> {
    pub(super) fn new(mut er: HtmlElementUpdater<'er, C>) -> Self {
        er.element_updater_mut().set_static_mode();
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementUpdater<'er, C> {
        self.0
    }
}

impl<'er, C: Component> StaticAttributes<'er, C> {
    pub(super) fn new(mut er: HtmlElementUpdater<'er, C>) -> Self {
        er.element_updater_mut().set_static_mode();
        Self(er)
    }
    pub(super) fn into_inner(self) -> HtmlElementUpdater<'er, C> {
        self.0
    }
    pub fn static_attributes_only(self) -> StaticAttributesOnly<'er, C> {
        StaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for AttributesOnly<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        self.0.element_updater()
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<C> {
        self.0.element_updater_mut()
    }
}
impl<'er, C: Component> HtmlElementUpdaterMut<C> for AttributesOnly<'er, C> {
    fn html_element_updater_mut(&mut self) -> &'er mut HtmlElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for StaticAttributesOnly<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        self.0.element_updater()
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<C> {
        self.0.element_updater_mut()
    }
}
impl<'er, C: Component> HtmlElementUpdaterMut<C> for StaticAttributesOnly<'er, C> {
    fn html_element_updater_mut(&mut self) -> &'er mut HtmlElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for StaticAttributes<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        self.0.element_updater()
    }
    fn element_updater_mut(&mut self) -> &mut ElementUpdater<C> {
        self.0.element_updater_mut()
    }
}
impl<'er, C: Component> HtmlElementUpdaterMut<C> for StaticAttributes<'er, C> {
    fn html_element_updater_mut(&mut self) -> &'er mut HtmlElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> MethodsForSelectedValueSelectedIndex<C> for HtmlElementUpdater<'er, C> {}
impl<'er, C: Component> HamsHandMade<C> for HtmlElementUpdater<'er, C> {}
impl<'er, C: Component> HamsForDistinctNames<C> for HtmlElementUpdater<'er, C> {}

impl<'er, C: Component> MethodsForSelectedValueSelectedIndex<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> HamsHandMade<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> HamsForDistinctNames<C> for StaticAttributes<'er, C> {}

impl<'er, C: Component> MethodsForSelectedValueSelectedIndex<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> HamsHandMade<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> HamsForDistinctNames<C> for AttributesOnly<'er, C> {}

impl<'er, C: Component> MethodsForSelectedValueSelectedIndex<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> HamsHandMade<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> HamsForDistinctNames<C> for StaticAttributesOnly<'er, C> {}

impl<'er, C: Component> MethodsForEvents<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> MethodsForEvents<C> for StaticAttributesOnly<'er, C> {}
impl<'er, C: Component> MethodsForEvents<C> for AttributesOnly<'er, C> {}
