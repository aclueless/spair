use super::ElementRenderMut;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::component::queue_render::Value;

make_traits_for_attribute_values! {
    BoolAttributeValue
    { bool, set_bool_attribute queue_bool_attribute }

    I32AttributeValue
    { i32, set_i32_attribute queue_i32_attribute }

    U32AttributeValue
    { u32, set_u32_attribute queue_u32_attribute }

    F64AttributeValue
    { f64, set_f64_attribute queue_f64_attribute }

    AttributeMinMax
    {
        f64, set_f64_attribute queue_f64_attribute
        &str, set_str_attribute queue_str_attribute
    }
    StringAttributeValue
    {
        // TODO: A spair::Value<T> only support for String, not &str or &String
        &str, set_str_attribute queue_str_attribute
        &String, set_str_attribute queue_str_attribute
        String, set_string_attribute queue_string_attribute
    }
}
