use crate::{component::{Component, queue_render::Value}, render::base::ElementRender, dom::{QrNormalAttribute, QrBoolAttribute, QrAttribute, ElementStatus}};

impl<'a, C: Component> ElementRender<'a, C> {
    pub fn queue_bool_attribute(&mut self, name: &'static str, value: &Value<bool>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let qra = QrBoolAttribute::new(self.comp(), element, unmounted, name);
        let index = self.index();
        self.element_mut().attribute_list_mut().store_queue_render(index, QrAttribute::new(Box::new(qra.clone())));
        self.next_index();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                qra.update(*this.value());
                this.add_render(Box::new(qra));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn queue_string_attribute(&mut self, name: &'static str, value: &Value<String>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let qra = QrNormalAttribute::new(self.comp(), element, unmounted, name);
        let index = self.index();
        self.element_mut().attribute_list_mut().store_queue_render(index, QrAttribute::new(Box::new(qra.clone())));
        self.next_index();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                qra.update(this.value());
                this.add_render(Box::new(qra));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn queue_attribute<T:'static + ToString>(&mut self, name: &'static str, value: &Value<T>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let qra = QrNormalAttribute::new(self.comp(), element, unmounted, name);
        let index = self.index();
        self.element_mut().attribute_list_mut().store_queue_render(index, QrAttribute::new(Box::new(qra.clone())));
        self.next_index();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                qra.update(&this.value().to_string());
                this.add_render(Box::new(qra));
            }
            Err(e) => log::error!("{}", e),
        }
    }
}
