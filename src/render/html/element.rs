use super::{AttributesOnly, StaticAttributes, StaticAttributesOnly};
use crate::{
    component::{Comp, Component},
    dom::{ElementType, WsElement},
    render::base::{ElementUpdater, ElementUpdaterMut, MethodsForEvents},
};
use wasm_bindgen::JsCast;

#[derive(Debug)]
enum SelectedOption {
    None,
    Value(String),
    Index(i32),
}

// This struct is used for <select> element to set the selected value
// It will do the setting on drop
pub struct SelectElementValueManager {
    // TODO: store an HtmlSelectElement here
    element: web_sys::Node,
    value: Option<SelectedOption>,
}

impl SelectElementValueManager {
    pub fn new(select_element: &web_sys::Node) -> Self {
        Self {
            element: select_element.clone(),
            value: None,
        }
    }
    pub fn set_selected_value(&mut self, value: Option<String>) {
        self.value = Some(
            value
                .map(SelectedOption::Value)
                .unwrap_or(SelectedOption::None),
        );
    }

    pub fn set_selected_index(&mut self, index: Option<i32>) {
        self.value = Some(
            index
                .map(SelectedOption::Index)
                .unwrap_or(SelectedOption::None),
        );
    }
}

impl Drop for SelectElementValueManager {
    fn drop(&mut self) {
        if let Some(selected_option) = self.value.take() {
            let select = self.element.unchecked_ref::<web_sys::HtmlSelectElement>();
            match selected_option {
                SelectedOption::None => select.set_selected_index(-1),
                SelectedOption::Value(value) => select.set_value(&value),
                SelectedOption::Index(index) => select.set_selected_index(index),
            }
        }
    }
}

pub trait HtmlElementUpdaterMut<'updater, C: Component> {
    fn html_element_updater_mut(&mut self) -> &mut HtmlElementUpdater<'updater, C>;
}

/// This struct helps rendering the element's attributes and its child nodes.
/// This will be exported as spair::Element to use as the root element of components.
/// Most of HTML attributes and HTML elements can be rendered using methods attached to this
/// struct (respectively via HamsForDistinctNames and HemsForDistinctNames). Some HTML attributes
/// and HTML elements that their names appear in both will not be call directly on this struct.
pub struct HtmlElementUpdater<'updater, C: Component> {
    element_updater: ElementUpdater<'updater, C>,
    select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'updater, C: Component> ElementUpdaterMut<'updater, C> for HtmlElementUpdater<'updater, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.element_updater
    }

    fn element_updater_mut(&mut self) -> &mut ElementUpdater<'updater, C> {
        &mut self.element_updater
    }
}

impl<'updater, C: Component> HtmlElementUpdaterMut<'updater, C>
    for HtmlElementUpdater<'updater, C>
{
    fn html_element_updater_mut(&mut self) -> &mut HtmlElementUpdater<'updater, C> {
        self
    }
}

impl<'updater, C: Component> From<ElementUpdater<'updater, C>> for HtmlElementUpdater<'updater, C> {
    fn from(element_updater: ElementUpdater<'updater, C>) -> Self {
        let select_element_value_manager: Option<SelectElementValueManager> =
            match element_updater.element().element_type() {
                ElementType::Select => Some(element_updater.element().ws_element().unchecked_ref()),
                _ => None,
            }
            .map(SelectElementValueManager::new);
        Self {
            element_updater,
            select_element_value_manager,
        }
    }
}
impl<'updater, C: Component> HtmlElementUpdater<'updater, C> {
    pub(super) fn into_parts(
        self,
    ) -> (
        ElementUpdater<'updater, C>,
        Option<SelectElementValueManager>,
    ) {
        (self.element_updater, self.select_element_value_manager)
    }

    pub fn state(&self) -> &'updater C {
        self.element_updater.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.element_updater.comp()
    }

    #[cfg(feature = "svg")]
    pub fn as_svg_element(self) -> crate::render::svg::SvgElementUpdater<'updater, C> {
        self.element_updater.into()
    }

    pub fn attributes_only(self) -> AttributesOnly<'updater, C> {
        AttributesOnly::new(self)
    }

    pub fn static_attributes_only(self) -> StaticAttributesOnly<'updater, C> {
        StaticAttributesOnly::new(self)
    }

    pub fn static_attributes(self) -> StaticAttributes<'updater, C> {
        StaticAttributes::new(self)
    }

    pub fn ws_element(&self) -> &WsElement {
        self.element_updater.element().ws_element()
    }

    fn set_selected_value_string(&mut self, value: Option<String>) {
        if let Some(manager) = self.select_element_value_manager.as_mut() {
            manager.set_selected_value(value);
        }
    }

    fn set_selected_value(&mut self, value: Option<&str>) {
        self.set_selected_value_string(value.map(ToString::to_string));
    }

    fn set_selected_index(&mut self, index: Option<i32>) {
        if let Some(manager) = self.select_element_value_manager.as_mut() {
            manager.set_selected_index(index);
        }
    }

    fn set_value(&mut self, value: &str) {
        if self.ws_element().set_value(value, false) {
            // It has no effect if you set a value for
            // a <select> element before adding its <option>s,
            // this hacking should finish in the list() method.
            // Is there a better solution?
            self.set_selected_value(Some(value));
        }
    }

    // This must call by a <select>
    pub(super) fn selected_value_str(&mut self, value: &str) {
        if self.element_updater.str_value_change(value).0 {
            self.set_value(value);
        }
    }

    // This must call by a <select>
    pub(super) fn selected_value_string(&mut self, value: String) {
        if self.element_updater.str_value_change(&value).0 {
            self.set_selected_value_string(Some(value));
        }
    }

    // This must call by a <select>
    pub(super) fn selected_value_optional_str(&mut self, value: Option<&str>) {
        match self.element_updater.element_mut().element_type() {
            ElementType::Select => {
                if self.element_updater.option_str_value_change(value).0 {
                    self.set_selected_value(value);
                }
            }
            _ => log::warn!("Should a value:Option<String> only can be set on a select element?"),
        }
    }

    // This must call by a <select>
    pub(super) fn selected_value_optional_string(&mut self, value: Option<String>) {
        if self
            .element_updater
            .option_str_value_change(value.as_deref())
            .0
        {
            self.set_selected_value_string(value);
        }
    }

    fn selected_index(&mut self, value: i32) {
        match self.element_updater.element_mut().element_type() {
            ElementType::Select => {
                if !self.element_updater.i32_value_change(value) {
                    return;
                }
                self.set_selected_index(Some(value));
            }
            _ => log::warn!("Should a selected_index only can be set on a select element?"),
        }
    }
    pub(super) fn selected_index_usize(&mut self, value: usize) {
        self.selected_index(value as i32);
    }
    pub(super) fn selected_index_optional_usize(&mut self, value: Option<usize>) {
        let value = value.map(|value| value as i32).unwrap_or(-1);
        self.selected_index(value);
    }
}

impl<'updater, C: Component> MethodsForEvents<'updater, C> for HtmlElementUpdater<'updater, C> {}

pub trait HemsWholeSelf<'updater, C: Component>:
    Sized + Sized + HtmlElementUpdaterMut<'updater, C>
{
    fn dangerously_set_inner_html(mut self, value: &str) {
        // Currently always set the value
        // TODO: Should we check for value change using some kind of hash?
        self.html_element_updater_mut()
            .ws_element()
            .as_ref()
            .set_inner_html(value);
    }
}

impl<'updater, C: Component> HemsWholeSelf<'updater, C> for HtmlElementUpdater<'updater, C> {}
impl<'updater, C: Component> HemsWholeSelf<'updater, C> for AttributesOnly<'updater, C> {}
impl<'updater, C: Component> HemsWholeSelf<'updater, C> for StaticAttributes<'updater, C> {}
impl<'updater, C: Component> HemsWholeSelf<'updater, C> for StaticAttributesOnly<'updater, C> {}
