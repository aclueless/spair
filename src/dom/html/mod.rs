pub mod attributes;
pub mod nodes;

pub struct HtmlUpdater<'a, C>(crate::dom::element::ElementUpdater<'a, C>);

impl<'a, C: crate::component::Component> crate::dom::attributes::AttributeSetter
    for HtmlUpdater<'a, C>
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
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0.store_listener(listener);
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        self.0.check_bool_attribute(value)
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        self.0.check_str_attribute(value)
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        self.0.check_i32_attribute(value)
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        self.0.check_u32_attribute(value)
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        self.0.check_f64_attribute(value)
    }

    fn set_selected_value(&mut self, value: Option<&str>) {
        self.0.set_selected_value(value)
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.0.set_selected_index(index)
    }
}

impl<'a, C: crate::component::Component> attributes::AttributeSetter<C> for HtmlUpdater<'a, C> where
    C: crate::component::Component
{
}
