use super::ElementRenderMut;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::queue_render::{MapValue, Value};

make_traits_for_attribute_values! {
    BoolAttributeValue
    { bool, set_bool_attribute queue_bool_attribute queue_bool_attribute_map, }

    I32AttributeValue
    { i32, set_i32_attribute queue_attribute queue_attribute_map, }

    U32AttributeValue
    { u32, set_u32_attribute queue_attribute queue_attribute_map, }

    F64AttributeValue
    { f64, set_f64_attribute queue_attribute queue_attribute_map, }

    AttributeMinMax
    {
        f64, set_f64_attribute queue_attribute queue_attribute_map,
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute queue_string_attribute queue_attribute_map,
    }
    StringAttributeValue
    {
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        &String, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute queue_string_attribute queue_attribute_map,
    }
}
