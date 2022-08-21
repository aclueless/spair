use crate::{
    component::Component,
    queue_render::{MapValue, Value},
    render::html::HtmlElementRender,
};
impl<'a, C: Component> HtmlElementRender<'a, C> {
    pub fn queue_attribute_value_string(&self, value: &Value<String>) {}
    pub fn queue_attribute_value_optional_string(&self, value: &Value<Option<String>>) {}
    pub fn queue_attribute_selected_index_usize(&self, value: &Value<usize>) {}
    pub fn queue_attribute_selected_index_optional_usize(&self, value: &Value<Option<usize>>) {}
    pub fn queue_attribute_value_string_map<T: 'static>(&self, value: MapValue<C, T, String>) {}
    pub fn queue_attribute_value_optional_string_map<T: 'static>(
        &self,
        value: MapValue<C, T, Option<String>>,
    ) {
    }
    pub fn queue_attribute_selected_index_usize_map<T: 'static>(
        &self,
        value: MapValue<C, T, usize>,
    ) {
    }
    pub fn queue_attribute_selected_index_optional_usize_map<T: 'static>(
        &self,
        value: MapValue<C, T, Option<usize>>,
    ) {
    }
}
