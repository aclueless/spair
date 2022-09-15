/// Make trait with methods to set attributes
macro_rules! make_trait_for_attribute_methods {
    (
        TestStructs: ($($TestStructName:ident)*)
        TraitName: $TraitName:ident
        attributes: $(
            $attribute_type:ident $method_name:ident $($attribute_name:literal)?
        )+
    ) => {
        make_test_methods!{
            TestStructs: ($($TestStructName)*)
            names: $($method_name)+
        }

        pub trait $TraitName<C: Component>: Sized + crate::render::base::BaseElementRenderMut<C>
        {$(
            make_trait_for_attribute_methods! {
                @each
                $method_name $($attribute_name)? => $attribute_type
            }
        )+}
    };
    (@each $method_name:ident => $attribute_type:ident) => {
        make_trait_for_attribute_methods! {
            @each
            $method_name stringify!($method_name) => $attribute_type
        }
    };
    (@each $method_name:ident $attribute_name:expr => bool) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => BoolAttributeValue
        }
    };
    (@each $method_name:ident $attribute_name:expr => u32) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => U32AttributeValue
        }
    };
    (@each $method_name:ident $attribute_name:expr => i32) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => I32AttributeValue
        }
    };
    (@each $method_name:ident $attribute_name:expr => f64) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => F64AttributeValue
        }
    };
    (@each $method_name:ident $attribute_name:expr => str) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => StringAttributeValue
        }
    };
    (@each $method_name:ident $attribute_name:expr => $attribute_type:ident) => {
        make_trait_for_attribute_methods! {
            @create_fn
            $method_name $attribute_name => $attribute_type
        }
    };
    (@create_fn $method_name:ident $attribute_name:expr => $AttributeValueTrait:ident) => {
        #[allow(clippy::wrong_self_convention)]
        fn $method_name(mut self, value: impl $AttributeValueTrait<C>) -> Self {
            value.render($attribute_name, &mut self);
            self
        }
    };
}

macro_rules! make_trait_for_attributes_with_predefined_values {
    (
        TestStructs: ($($TestStructName:ident)*)
        TraitName: $TraitName:ident
        $({
            $AttributeValueTrait:ident
            $AttributeValueType:ident {
                $(
                    $(#[$variant_meta_content:meta])*
                    $AttributeValueVariants:ident => $attribute_value_str:literal,
                )+
            }
            {
                $(
                    $attribute_method_name:ident $($attribute_name_str:literal)?
                )*
            }
        })+
    ) => {
        make_trait_for_attributes_with_predefined_values!(
            @impl_test
            TestStructs: ($($TestStructName)*)
            $({
                names: $($attribute_method_name)*
            })+
        );
        pub trait $TraitName<C: Component>: Sized + crate::render::base::BaseElementRenderMut<C> {
            $(
            $(
                make_trait_for_attributes_with_predefined_values!(
                    @each_fn
                    $AttributeValueTrait
                    $attribute_method_name
                    $($attribute_name_str)?
                );
            )*
            )+
        }
        $(
            make_trait_for_attributes_with_predefined_values!(
                @each_type
                $AttributeValueTrait
                $AttributeValueType {
                $(
                    $(#[$variant_meta_content])*
                    $AttributeValueVariants => $attribute_value_str
                )+
                }
            );
        )+
        pub mod predefined_attribute_types {
            $(
                pub use super::$AttributeValueType;
            )+
        }
    };
    (
        @impl_test
        TestStructs: ()
        $({
            names: $($name:ident)*
        })+
    ) => {
    };
    (
        @impl_test
        TestStructs: ($TestStructName:ident $($MoreTestStructName:ident)*)
        $({
            names: $($name:ident)*
        })+
    ) => {
        $(
             make_test_methods!{
                TestStructs: ($TestStructName)
                names: $($name)*
            }
        )+
        make_trait_for_attributes_with_predefined_values!(
            @impl_test
            TestStructs: ($($MoreTestStructName)*)
            $({
                names: $($name)*
            })+
        );
    };
    (
        @each_type
        $AttributeValueTrait:ident
        $AttributeValueType:ident {
            $(
                $(#[$variant_meta_content:meta])*
                $AttributeValueVariants:ident => $attribute_value_str:literal
            )+
        }
    ) => {
        pub enum $AttributeValueType {
            $(
                $(#[$variant_meta_content])*
                $AttributeValueVariants,
            )+
        }

        impl $AttributeValueType {
            fn as_str(&self) -> &str {
                match self {
                    $(
                        #[allow(deprecated)]
                        Self::$AttributeValueVariants => $attribute_value_str,
                    )+
                }
            }
        }

        pub trait $AttributeValueTrait<C: Component> {
            fn render(self, attribute_name: &str, element: &mut crate::render::base::BaseElementRender<C>);
        }

        impl<C: Component> $AttributeValueTrait<C> for $AttributeValueType {
            fn render(self, attribute_name: &str, element: &mut crate::render::base::BaseElementRender<C>) {
                element.set_str_attribute(attribute_name, self.as_str());
            }
        }
    };
    (
        @each_fn
        $AttributeValueTrait: ident
        $attribute_method_name: ident
    ) => {
        make_trait_for_attributes_with_predefined_values!(
            @each_fn
            $AttributeValueTrait
            $attribute_method_name
            stringify!($attribute_method_name)
        );
    };
    (
        @each_fn
        $AttributeValueTrait: ident
        $attribute_method_name: ident
        $attribute_name_str:expr
    ) => {
        fn $attribute_method_name(mut self, value: impl $AttributeValueTrait<C>) -> Self {
            value.render($attribute_name_str, self.element_render_mut());
            self
        }
    };
}

macro_rules! make_traits_for_attribute_values {
    (
        $(
            $AttributeTrait:ident {
                $($attribute_type:ty, $method_name:ident $queue_render_method_name:ident $queue_render_method_name_map:ident,)+
            }
        )+
    ) => {
        $(
            pub trait $AttributeTrait<C: Component> {
                fn render(self, name: &'static str, element: impl crate::render::base::BaseElementRenderMut<C>);
            }
            $(
                impl<C: Component> $AttributeTrait<C> for $attribute_type {
                    fn render(self, name: &'static str, mut element: impl crate::render::base::BaseElementRenderMut<C>) {
                        element.element_render_mut().$method_name(name, self);
                    }
                }
                make_traits_for_attribute_values! {
                    @each_queue_render
                    $AttributeTrait
                    $attribute_type, $queue_render_method_name $queue_render_method_name_map
                }
            )+
        )+
    };
    (
        @each_queue_render
        $AttributeTrait:ident
        $attribute_type:ty, NO_QUEUE_RENDER NO_QUEUE_RENDER
    ) => {
    };
    (
        @each_queue_render
        $AttributeTrait:ident
        $attribute_type:ty, $queue_render_method_name:ident $queue_render_method_name_map:ident
    ) => {
        #[cfg(feature = "queue-render")]
        impl<C: Component> $AttributeTrait<C> for &QrVal<$attribute_type> {
            fn render(self, name: &'static str, mut element: impl crate::render::base::BaseElementRenderMut<C>) {
                element.element_render_mut().$queue_render_method_name(name, self);
            }
        }
        #[cfg(feature = "queue-render")]
        impl<C: Component, T: 'static> $AttributeTrait<C> for MapValue<C, T, $attribute_type> {
            fn render(self, name: &'static str, mut element: impl crate::render::base::BaseElementRenderMut<C>) {
                element.element_render_mut().$queue_render_method_name_map(name, self);
            }
        }
    };
}
