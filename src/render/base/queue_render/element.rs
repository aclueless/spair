use crate::{component::{Component, queue_render::Value}, render::base::ElementRender};

impl<'a, C: Component> ElementRender<'a, C> {
    pub fn queue_bool_attribute(&mut self, name: &str, value: &Value<bool>) {
        //self.element.set_bool_attribute(name, value);
        //value.
    }

    pub fn queue_str_attribute(&mut self, name: &str, value: &Value<&str>) {
        //self.element.set_str_attribute(name, value);
    }

    pub fn queue_string_attribute(&mut self, name: &str, value: &Value<String>) {
        //self.element.set_str_attribute(name, value);
    }

    pub fn queue_i32_attribute(&mut self, name: &str, value: &Value<i32>) {
        //self.element.set_i32_attribute(name, value);
    }

    pub fn queue_u32_attribute(&mut self, name: &str, value: &Value<u32>) {
        //self.element.set_u32_attribute(name, value);
    }

    pub fn queue_f64_attribute(&mut self, name: &str, value: &Value<f64>) {
        //self.element.set_f64_attribute(name, value);
    }
}
