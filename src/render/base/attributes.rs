use super::ElementUpdater;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::queue_render::value::{MapValue, QrVal};

make_traits_for_attribute_values! {
    BoolAttributeValue
    { bool, set_bool_attribute qr_attribute qrm_attribute, }

    I32AttributeValue
    { i32, set_i32_attribute qr_attribute qrm_attribute, }

    U32AttributeValue
    { u32, set_u32_attribute qr_attribute qrm_attribute, }

    F64AttributeValue
    { f64, set_f64_attribute qr_attribute qrm_attribute, }

    AttributeMinMax
    {
        f64, set_f64_attribute qr_attribute qrm_attribute,
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute qr_attribute qrm_attribute,
    }
    StringAttributeValue
    {
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        &String, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute qr_attribute qrm_attribute,
    }
}

pub trait Class<C: Component> {
    fn render(self, element: &mut ElementUpdater<C>);
}

impl<C: Component> Class<C> for &str {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.class(self);
    }
}

impl<C: Component> Class<C> for String {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.class(&self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component> Class<C> for &QrVal<String> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qr_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for MapValue<C, T, String> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrm_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for MapValue<C, T, &'static str> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrm_str_class(self);
    }
}
