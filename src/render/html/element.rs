use super::{AttributesOnly, StaticAttributes, StaticAttributesOnly};
use crate::{
    component::{Comp, Component},
    dom::{AttributeValueList, ElementType, WsElement},
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

pub trait HtmlElementUpdaterMut<C: Component> {
    fn html_element_render_mut(&mut self) -> &mut HtmlElementUpdater<C>;
}

/// This struct helps rendering the element's attributes and its child nodes.
/// This will be exported as spair::Element to use as the root element of components.
/// Most of HTML attributes and HTML elements can be rendered using methods attached to this
/// struct (respectively via HamsForDistinctNames and HemsForDistinctNames). Some HTML attributes
/// and HTML elements that their names appear in both will not be call directly on this struct.
pub struct HtmlElementUpdater<'er, C: Component> {
    element_render: ElementUpdater<'er, C>,
    select_element_value_manager: Option<SelectElementValueManager>,
}

impl<'er, C: Component> ElementUpdaterMut<C> for HtmlElementUpdater<'er, C> {
    fn element_render(&self) -> &ElementUpdater<C> {
        &self.element_render
    }

    fn element_render_mut(&mut self) -> &'er mut ElementUpdater<C> {
        &mut self.element_render
    }
}

impl<'er, C: Component> HtmlElementUpdaterMut<C> for HtmlElementUpdater<'er, C> {
    fn html_element_render_mut(&mut self) -> &'er mut HtmlElementUpdater<C> {
        self
    }
}

impl<'er, C: Component> From<ElementUpdater<'er, C>> for HtmlElementUpdater<'er, C> {
    fn from(element_render: ElementUpdater<'er, C>) -> Self {
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
impl<'er, C: Component> HtmlElementUpdater<'er, C> {
    pub(super) fn into_parts(self) -> (ElementUpdater<'er, C>, Option<SelectElementValueManager>) {
        (self.element_render, self.select_element_value_manager)
    }

    pub fn state(&self) -> &'er C {
        self.element_render.state()
    }

    pub fn comp(&self) -> Comp<C> {
        self.element_render.comp()
    }

    #[cfg(feature = "svg")]
    pub fn as_svg_element(self) -> crate::render::svg::SvgElementUpdater<'er, C> {
        self.element_render.into()
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

    pub fn ws_element(&self) -> &WsElement {
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
        if self.ws_element().set_value(value, false) {
            // It has no effect if you set a value for
            // a <select> element before adding its <option>s,
            // this hacking should finish in the list() method.
            // Is there a better solution?
            self.set_selected_value(Some(value));
        }
    }

    pub(super) fn selected_value_str(&mut self, value: &str) {
        if !self
            .element_render
            .must_render_attribute(value, AttributeValueList::check_str_attribute)
        {
            return;
        }
        self.set_value(value);
    }
    pub(super) fn selected_value_string(&mut self, value: String) {
        if !self
            .element_render
            .must_render_attribute(value.as_str(), AttributeValueList::check_str_attribute)
        {
            return;
        }
        self.set_value(&value);
    }
    pub(super) fn selected_value_optional_str(&mut self, value: Option<&str>) {
        match self.element_render.element_mut().element_type() {
            ElementType::Select => {
                if !self
                    .element_render
                    .must_render_attribute(value, AttributeValueList::check_optional_str_attribute)
                {
                    return;
                }
                self.set_selected_value(value);
            }
            _ => log::warn!("Should a value:Option<String> only can be set on a select element?"),
        }
    }
    pub(super) fn selected_value_optional_string(&mut self, value: Option<String>) {
        self.selected_value_optional_str(value.as_deref());
    }

    fn selected_index(&mut self, value: i32) {
        match self.element_render.element_mut().element_type() {
            ElementType::Select => {
                if !self
                    .element_render
                    .must_render_attribute(value, AttributeValueList::check_i32_attribute)
                {
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

impl<'er, C: Component> MethodsForEvents<C> for HtmlElementUpdater<'er, C> {}
