use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub struct SvgStaticAttributes<'a, C>(super::SvgUpdater<'a, C>);

impl<'a, C: crate::component::Component> SvgStaticAttributes<'a, C> {
    pub(super) fn new(su: super::SvgUpdater<'a, C>) -> Self {
        Self(su)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}
}

mod sealed {
    // TODO: Copied from dom::attributes. Should be a common trait?
    use wasm_bindgen::{JsCast, UnwrapThrowExt};

    pub trait AttributeSetter {
        fn ws_html_element(&self) -> &web_sys::HtmlElement;
        fn ws_element(&self) -> &web_sys::Element;
        fn require_set_listener(&mut self) -> bool;
        fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>);

        // Check if the attribute need to be set (and store the new value for the next check)
        fn check_bool_attribute(&mut self, value: bool) -> bool;
        fn check_str_attribute(&mut self, value: &str) -> bool;
        fn check_i32_attribute(&mut self, value: i32) -> bool;
        fn check_u32_attribute(&mut self, value: u32) -> bool;
        fn check_f64_attribute(&mut self, value: f64) -> bool;

        fn set_bool_attribute(&mut self, name: &str, value: bool) {
            if self.check_bool_attribute(value) {
                if value {
                    self.ws_element()
                        .set_attribute(name, "")
                        .expect_throw("Unable to set bool attribute");
                } else {
                    self.ws_element()
                        .remove_attribute(name)
                        .expect_throw("Unable to remove bool attribute");
                }
            }
        }

        fn set_str_attribute(&mut self, name: &str, value: &str) {
            if self.check_str_attribute(value) {
                self.ws_element()
                    .set_attribute(name, value)
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_i32_attribute(&mut self, name: &str, value: i32) {
            if self.check_i32_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_u32_attribute(&mut self, name: &str, value: u32) {
            if self.check_u32_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_f64_attribute(&mut self, name: &str, value: f64) {
            if self.check_f64_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }
    }
}

pub trait SvgAttributeSetter<C>: Sized + sealed::AttributeSetter {
    create_methods_for_attributes! {
        f64     x
        f64     y
        f64     r
    }
}

impl<'a, C: crate::component::Component> SvgAttributeSetter<C> for super::SvgUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> sealed::AttributeSetter for super::SvgUpdater<'a, C> {
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.element.ws_element.unchecked_ref()
    }

    fn ws_element(&self) -> &web_sys::Element {
        &self.element.ws_element
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.element.attributes.store_listener(self.index, listener);
        self.index += 1;
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        let rs = self
            .element
            .attributes
            .check_bool_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        let rs = self
            .element
            .attributes
            .check_str_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        let rs = self
            .element
            .attributes
            .check_i32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        let rs = self
            .element
            .attributes
            .check_u32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        let rs = self
            .element
            .attributes
            .check_f64_attribute(self.index, value);
        self.index += 1;
        rs
    }
}
