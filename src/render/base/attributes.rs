use super::ElementRenderMut;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::component::queue_render::Value;

macro_rules! make_attribute_value_traits {
    (
        $(
            $new:tt $AttributeTrait:ident $attribute_type:ty, $method_name:ident $queue_method_name:ident
        )+
    ) => {
        $(
            make_attribute_value_traits!(@each $new $AttributeTrait $attribute_type, $method_name $queue_method_name);
        )+
    };
    (@each new $AttributeTrait:ident $attribute_type:ty, $method_name:ident $queue_method_name:ident) => {
        pub trait $AttributeTrait<C: Component> {
            fn render(self, naem: &str, node: impl ElementRenderMut<C>);
        }
        make_attribute_value_traits!(@each $AttributeTrait $attribute_type, $method_name $queue_method_name);
    };
    (@each old $AttributeTrait:ident $attribute_type:ty, $method_name:ident $queue_method_name:ident) => {
        make_attribute_value_traits!(@each $AttributeTrait $attribute_type, $method_name $queue_method_name);
    };
    (@each $AttributeTrait:ident $attribute_type:ty, $method_name:ident $queue_method_name:ident) => {
        impl<C: Component> $AttributeTrait<C> for $attribute_type {
            fn render(self, name: &str, mut node: impl ElementRenderMut<C>) {
                node.element_render_mut().$method_name(name, self);
            }
        }
        #[cfg(feature = "queue-render")]
        impl<C: Component> $AttributeTrait<C> for &Value<$attribute_type> {
            fn render(self, name: &str, mut node: impl ElementRenderMut<C>) {
                node.element_render_mut().$queue_method_name(name, self);
            }
        }
    };
}

make_attribute_value_traits! {
    new BoolAttributeValue  bool, set_bool_attribute queue_bool_attribute
    new I32AttributeValue   i32, set_i32_attribute queue_i32_attribute
    new U32AttributeValue   u32, set_u32_attribute queue_u32_attribute
    new F64AttributeValue   f64, set_f64_attribute queue_f64_attribute
    new AttributeMinMax     f64, set_f64_attribute queue_f64_attribute
    old AttributeMinMax     &str, set_str_attribute queue_str_attribute
    new StringAttributeValue &str, set_str_attribute queue_str_attribute
}
