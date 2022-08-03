/// This macro make methods on the given struct to find conflicting names.
/// It is to use to find elements and attributes that have the same names.
/// If the element/attribute names are unique, then it can be define directly
/// on the mostly use render which users can conviniently access the methods.
/// Otherwise, it must be define on a separated struct, and user have to switch
/// to that struct before accessing the methods.
macro_rules! make_test_methods {
    (
        TestStructs: ($($TestStructName:ident)*)
        names: $($name:ident)*
    ) => {
        make_test_methods!(
            @impl_for_first_struct
            ($($TestStructName)*)
            $($name)*
        );
    };
    (
        @impl_for_first_struct
        ()
        $($name:ident)*
    ) => {
        // No more struct, make_test_methods! done
    };
    (
        @impl_for_first_struct
        ($TestStructName:ident $($MoreTestStructName:ident)*)
        $($name:ident)*
    ) => {
        #[cfg(test)]
        impl $TestStructName {
            $(
                #[allow(dead_code)]
                fn $name() {}
            )*
        }
        make_test_methods!(
            @impl_for_first_struct
            ($($MoreTestStructName)*)
            $($name)*
        );
    }
}

/// This macro just implements no-op methods witch deprecated attributes attached to them.
/// The deprecated attribute provides a useful message to remind user what what to do.
macro_rules! make_trait_for_deprecated_methods {
    (
        TestStructs: ($($TestStructName:ident)*)
        $(#[$doc:meta])*
        TraitName: $TraitName:ident
        names: $($name:ident)+
    ) => {
        make_test_methods!{
            TestStructs: ($($TestStructName)*)
            names: $($name)+
        }

        $(#[$doc])*
        pub trait $TraitName: Sized {$(
            #[doc = "This name is used for both attribute and element. Use `.attributes_only()."]
            #[doc = stringify!($name)]
            #[doc = "()` or `.static_attributes_only()."]
            #[doc = stringify!($name)]
            #[doc = "()` if you want to set an attribute. Or, use `.update_nodes()."]
            #[doc = stringify!($name)]
            #[doc = "()` or `.static_nodes()."]
            #[doc = stringify!($name)]
            #[doc = "()` if you want to render an element."]
            #[deprecated = "Call this method after `.attributes_only()`, `.static_attributes_only()`, `.update_nodes()`, `.static_nodes()`. See the documentation of this method for more information."]
            fn $name(self, _error_this_method_name_is_used_for_both_element_and_attribute: crate::render::AmbiguousName) -> Self {self}
            //fn $name(self) -> Self {self}
        )+}
    };
}

macro_rules! make_trait_for_element_methods {
    (
        TestStructs: ($($TestStructName:ident)*)
        TraitName: $TraitName:ident
        RenderElementTraitName: $RenderElementTraitName:ident
        ElementRenderType: $ElementRenderType:ident
        elements: $(
            $method_name:ident $($element_name:literal)?
        )+
    ) => {
         make_test_methods!{
            TestStructs: ($($TestStructName)*)
            names: $($method_name)+
        }

        pub trait $TraitName<C: Component>: Sized + $RenderElementTraitName<C, Self::Output> {
            type Output: From<Self> + NodeListRenderMut<C>;
            $(
            // fn $tag(self, element_render: impl FnOnce($ElementRenderType<C>)) -> Self::Output {
            //     self.render_element(stringify!($tag), element_render)
            // }
                make_trait_for_element_methods!(
                    @each_element
                    $ElementRenderType
                    $method_name
                    $($element_name)?
                );
            )+
        }
    };
    (
        @each_element
        $ElementRenderType:ident
        $method_name:ident
    ) => {
        make_trait_for_element_methods!(
            @each_element
            $ElementRenderType
            $method_name
            stringify!($method_name)
        );
    };
    (
        @each_element
        $ElementRenderType:ident
        $method_name:ident
        $element_name:expr
    ) => {
        fn $method_name(self, element_render: impl FnOnce($ElementRenderType<C>)) -> Self::Output {
            self.render_element($element_name, element_render)
        }
    };
}

macro_rules! make_trait_for_same_name_attribute_and_element_methods {
    (
        TestStructs: ($($TestStructName:ident)+)
        DeprecatedTraitName: $DeprecatedTraitName:ident
        for_elements {
            TraitName: $ElementTraitName:ident
            RenderElementTraitName: $RenderElementTraitName:ident
            ElementRenderType: $ElementRenderType:ident
        }
        for_attributes {
            //TraitDefinitionTokens: ($($TraitDefinitionTokens:tt)+)
            TraitName: $AttributeTraitName:ident
        }
        //method_names: $($name:ident)+
        ambiguous_attributes: $(
            $attribute_type:ident $method_name:ident $($attribute_name:literal)?
        )+
    ) => {
        make_test_methods!{
            TestStructs: ($($TestStructName)+)
            names: $($method_name)+
        }
        make_trait_for_deprecated_methods!{
            TestStructs: ()
            TraitName: $DeprecatedTraitName
            names: $($method_name)+
        }
        make_trait_for_element_methods! {
            TestStructs: ()
            TraitName: $ElementTraitName
            RenderElementTraitName: $RenderElementTraitName
            ElementRenderType: $ElementRenderType
            elements: $($method_name)+
        }
        make_trait_for_attribute_methods! {
            TestStructs: ()
            //TraitDefinitionTokens: ($($TraitDefinitionTokens)+)
            TraitName: $AttributeTraitName
            attributes: $(
                $attribute_type $method_name $($attribute_name)?
            )+
        }
    }
}
