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

        pub trait $TraitName<'updater, C: Component>: Sized + crate::render::base::ElementUpdaterMut<'updater, C>
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
        pub trait $TraitName<'updater, C: Component>: Sized + crate::render::base::ElementUpdaterMut<'updater, C> {
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
            // This "allow(dead_code)" is just to hide the warnings.
            // The reason is I want the current code to pass the test, but I don't have time to
            // look up why this is not use?
            // At the time of adding this, I plan to do some experiments of some newish ideas for
            // spair. If the experiments succeed, the current code may end up being removed entirely
            // from spair. Ortherwise, this "allow(dead_code)" must have some investigations!!!
            #[allow(dead_code)]
            fn render(self, attribute_name: &str, element: &mut crate::render::base::ElementUpdater<C>);
        }

        impl<C: Component> $AttributeValueTrait<C> for $AttributeValueType {
            fn render(self, attribute_name: &str, element: &mut crate::render::base::ElementUpdater<C>) {
                element.attribute(attribute_name, self.as_str());
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
            value.render($attribute_name_str, self.element_updater_mut());
            self
        }
    };
}

macro_rules! make_traits_for_attribute_values {
    (
        $(
            $AttributeTrait:ident {$(
                $attribute_type:ty:  $incremental_render:ident $queue_render:ident,
            )+}
        )+
    ) => {
        $(
            pub trait $AttributeTrait<C: Component> {
                fn render<'a>(self, name: &'static str, element: impl crate::render::base::ElementUpdaterMut<'a, C>);
            }
            $(
                make_traits_for_attribute_values! {
                    @each_incremental_render
                    $AttributeTrait
                    $attribute_type: $incremental_render
                }

                make_traits_for_attribute_values! {
                    @each_queue_render
                    $AttributeTrait
                    $attribute_type: $queue_render
                }
            )+
        )+
    };
    (@each_incremental_render $AttributeTrait:ident $attribute_type:ty: NO) => { };
    (@each_queue_render $AttributeTrait:ident $attribute_type:ty: NO) => { };
    (@each_incremental_render $AttributeTrait:ident $attribute_type:ty: YES) => {
        impl<C: Component> $AttributeTrait<C> for $attribute_type {
            fn render<'a>(self, name: &'static str, mut element: impl crate::render::base::ElementUpdaterMut<'a, C>) {
                element.element_updater_mut().attribute(name, self);
            }
        }
   };
    (@each_queue_render $AttributeTrait:ident $attribute_type:ty: YES) => {
        #[cfg(feature = "queue-render")]
        impl<C: Component> $AttributeTrait<C> for &crate::queue_render::val::QrVal<$attribute_type> {
            fn render<'a>(self, name: &'static str, mut element: impl crate::render::base::ElementUpdaterMut<'a, C>) {
                element.element_updater_mut().qr_attribute(name, self);
            }
        }
        #[cfg(feature = "queue-render")]
        impl<C: Component, T: 'static> $AttributeTrait<C> for crate::queue_render::val::QrValMap<T, $attribute_type> {
            fn render<'a>(self, name: &'static str, mut element: impl crate::render::base::ElementUpdaterMut<'a, C>) {
                element.element_updater_mut().qrm_attribute(name, self);
            }
        }
        #[cfg(feature = "queue-render")]
        impl<C: Component, T: 'static> $AttributeTrait<C> for crate::queue_render::val::QrValMapWithState<C, T, $attribute_type> {
            fn render<'a>(self, name: &'static str, mut element: impl crate::render::base::ElementUpdaterMut<'a, C>) {
                element.element_updater_mut().qrmws_attribute(name, self);
            }
        }
    };
}
