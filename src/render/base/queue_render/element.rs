use crate::{component::{Component, queue_render::Value}, render::base::ElementRender, dom::QrNormalAttribute};

impl<'a, C: Component> ElementRender<'a, C> {
    pub fn queue_bool_attribute(&mut self, name: &'static str, value: &Value<bool>) {
        //self.element.set_bool_attribute(name, value);
        //self.
    }

    pub fn queue_string_attribute(&mut self, name: &'static str, value: &Value<String>) {
        let element = self.element().ws_element().clone();
        let dropped = self.element().dropped();
        let qra = QrNormalAttribute::new(self.comp(), element, dropped, name);
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                qra.update(this.value());
                this.add_render(Box::new(qra));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    fn qra<T:'static + ToString>(&mut self, name: &'static str, value: &Value<T>) {
        let element = self.element().ws_element().clone();
        let dropped = self.element().dropped();
        let qra = QrNormalAttribute::new(self.comp(), element, dropped, name);
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                qra.update(&this.value().to_string());
                this.add_render(Box::new(qra));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn queue_i32_attribute(&mut self, name: &'static str, value: &Value<i32>) {
        self.qra(name, value);
    }

    pub fn queue_u32_attribute(&mut self, name: &'static str, value: &Value<u32>) {
        self.qra(name, value);
    }

    pub fn queue_f64_attribute(&mut self, name: &'static str, value: &Value<f64>) {
        self.qra(name, value);
    }
}
