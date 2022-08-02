use super::{
    AttributesOnly, MethodsForDeprecatingAttributesAndElementsWithAmbiguousNames,
    MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement, MethodsForHtmlAttributes,
    MethodsForHtmlAttributesWithPredifinedValues, MethodsForSpecialHtmlAttributes,
    StaticAttributes, StaticAttributesOnly,
};
use crate::component::{Comp, Component};
use crate::dom::ElementType;
use crate::render::base::{ElementRender, ElementRenderMut, MethodsForEvents};
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

pub trait HtmlElementRenderMut<C: Component> {
    fn html_element_render_mut(&mut self) -> &mut HtmlElementRender<C>;
}

/// This struct helps rendering the element's attributes and its child nodes.
/// This will be exported as spair::Element to use as the root element of components.
/// Most of HTML attributes and HTML elements can be rendered using methods attached to this
/// struct (respectively via MethodsForHtmlAttributes and HtmlElementMethods). Some HTML attributes
/// and HTML elements that their names appear in both will not be call directly on this struct.
pub struct HtmlElementRender<'er, C: Component> {
    element_render: ElementRender<'er, C>,
    select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'er, C: Component> ElementRenderMut<C> for HtmlElementRender<'er, C> {
    fn element_render(&self) -> &ElementRender<C> {
        &self.element_render
    }

    fn element_render_mut(&mut self) -> &'er mut ElementRender<C> {
        &mut self.element_render
    }
}

impl<'er, C: Component> HtmlElementRenderMut<C> for HtmlElementRender<'er, C> {
    fn html_element_render_mut(&mut self) -> &'er mut HtmlElementRender<C> {
        self
    }
}

impl<'er, C: Component> From<ElementRender<'er, C>> for HtmlElementRender<'er, C> {
    fn from(element_render: ElementRender<'er, C>) -> Self {
        let select_element_value_manager: Option<SelectElementValueManager> =
            match element_render.element().element_type() {
                ElementType::Select => Some(element_render.element().ws_element().unchecked_ref()),
                _ => None,
            }
            .map(SelectElementValueManager::new);
        Self {
            element_render,
            select_element_value_manager,
        }
    }
}
impl<'er, C: Component> HtmlElementRender<'er, C> {
    pub(super) fn into_parts(self) -> (ElementRender<'er, C>, Option<SelectElementValueManager>) {
        (self.element_render, self.select_element_value_manager)
    }

    pub fn state(&self) -> &'er C {
        self.element_render.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.element_render.comp()
    }

    pub fn attributes_only(self) -> AttributesOnly<'er, C> {
        AttributesOnly::new(self)
    }

    pub fn static_attributes_only(self) -> StaticAttributesOnly<'er, C> {
        StaticAttributesOnly::new(self)
    }

    pub fn static_attributes(self) -> StaticAttributes<'er, C> {
        StaticAttributes::new(self)
    }

    pub fn ws_element(&self) -> &web_sys::Element {
        self.element_render.element().ws_element()
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
        let element = self.element_render.element_mut();
        match element.element_type() {
            ElementType::Input => {
                let input = element
                    .ws_element()
                    .unchecked_ref::<web_sys::HtmlInputElement>();
                input.set_value(value);
            }
            ElementType::Select => {
                // It has no effect if you set a value for
                // a <select> element before adding its <option>s,
                // the hacking should finish in the list() method.
                // Is there a better solution?
                self.set_selected_value(Some(value));
            }
            ElementType::TextArea => {
                let text_area = element
                    .ws_element()
                    .unchecked_ref::<web_sys::HtmlTextAreaElement>();
                text_area.set_value(value);
            }
            ElementType::Option => {
                let option = element
                    .ws_element()
                    .unchecked_ref::<web_sys::HtmlOptionElement>();
                option.set_value(value);
            }
            ElementType::Other => {
                log::warn!(
                    ".value() is called on an element that is not <input>, <select>, <option>, <textarea>"
                );
            }
        }
    }

    pub(super) fn attribute_value_str(&mut self, value: &str) {
        if self
            .element_render
            .need_to_render_attribute(|al, index| al.check_str_attribute(index, value))
            == false
        {
            return;
        }
        self.set_value(value);
    }
    pub(super) fn attribute_value_string(&mut self, value: String) {
        if self
            .element_render
            .need_to_render_attribute(|al, index| al.check_str_attribute(index, &value))
            == false
        {
            return;
        }
        self.set_value(&value);
    }
    pub(super) fn attribute_value_optional_str(&mut self, value: Option<&str>) {
        match self.element_render.element_mut().element_type() {
            ElementType::Select => {
                if self.element_render.need_to_render_attribute(|al, index| {
                    al.check_optional_str_attribute(index, value)
                }) == false
                {
                    return;
                }
                self.set_selected_value(value);
            }
            _ => log::warn!("Should a value:Option<String> only can be set on a select element?"),
        }
    }
    pub(super) fn attribute_value_optional_string(&mut self, value: Option<String>) {
        self.attribute_value_optional_str(value.as_deref());
    }
    fn attribute_selected_index(&mut self, value: i32) {
        match self.element_render.element_mut().element_type() {
            ElementType::Select => {
                if self
                    .element_render
                    .need_to_render_attribute(|al, index| al.check_i32_attribute(index, value))
                    == false
                {
                    return;
                }
                self.set_selected_index(Some(value));
            }
            _ => log::warn!("Should a selected_index only can be set on a select element?"),
        }
    }
    pub(super) fn attribute_selected_index_usize(&mut self, value: usize) {
        self.attribute_selected_index(value as i32);
    }
    pub(super) fn attribute_selected_index_optional_usize(&mut self, value: Option<usize>) {
        let value = value.map(|value| value as i32).unwrap_or(-1);
        self.attribute_selected_index(value);
    }
}

impl<'er, C: Component> MethodsForDeprecatingAttributesAndElementsWithAmbiguousNames
    for HtmlElementRender<'er, C>
{
}
impl<'er, C: Component> MethodsForHtmlAttributes<C> for HtmlElementRender<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributeValueAndIndexOnHtmlSelectElement<C>
    for HtmlElementRender<'er, C>
{
}
impl<'er, C: Component> MethodsForSpecialHtmlAttributes<C> for HtmlElementRender<'er, C> {}
impl<'er, C: Component> MethodsForHtmlAttributesWithPredifinedValues<C>
    for HtmlElementRender<'er, C>
{
}

impl<'er, C: Component> MethodsForEvents<C> for HtmlElementRender<'er, C> {}
