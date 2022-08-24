use super::ElementRenderMut;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::queue_render::{MapValue, Value};

make_traits_for_attribute_values! {
    BoolAttributeValue
    { bool, set_bool_attribute qr_bool_attribute qrm_bool_attribute, }

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
        String, set_string_attribute qr_string_attribute qrm_attribute,
    }
    StringAttributeValue
    {
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        &String, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute qr_string_attribute qrm_attribute,
    }
}
