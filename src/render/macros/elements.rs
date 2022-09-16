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
            #[deprecated = "Call this method after `.attributes_only()`, `.static_attributes_only()`, `.update_nodes()`, `.static_nodes()` to disambiguate. See the documentation of this method for more information."]
            fn $name(self, _see_the_deprecation_note_for_method_document_for_information: crate::render::SeeDeprecationNoteOrMethodDocForInformation) -> Self {self}
        )+}
    };
}

macro_rules! make_trait_for_element_methods {
    (
        TestStructs: ($($TestStructName:ident)*)
        TraitName: $TraitName:ident
        UpdateElementTraitName: $UpdateElementTraitName:ident
        ElementUpdaterType: $ElementUpdaterType:ident
        elements: $(
            $method_name:ident $($element_name:literal)?
        )+
    ) => {
         make_test_methods!{
            TestStructs: ($($TestStructName)*)
            names: $($method_name)+
        }

        pub trait $TraitName<C: Component>: Sized + $UpdateElementTraitName<C, Self::Output> {
            type Output: From<Self> + NodesUpdaterMut<C>;
            $(
            // fn $tag(self, element_render: impl FnOnce($ElementUpdaterType<C>)) -> Self::Output {
            //     self.render_element(stringify!($tag), element_render)
            // }
                make_trait_for_element_methods!(
                    @each_element
                    $ElementUpdaterType
                    $method_name
                    $($element_name)?
                );
            )+
        }
    };
    (
        @each_element
        $ElementUpdaterType:ident
        $method_name:ident
    ) => {
        make_trait_for_element_methods!(
            @each_element
            $ElementUpdaterType
            $method_name
            stringify!($method_name)
        );
    };
    (
        @each_element
        $ElementUpdaterType:ident
        $method_name:ident
        $element_name:expr
    ) => {
        fn $method_name(self, element_render: impl FnOnce($ElementUpdaterType<C>)) -> Self::Output {
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
            UpdateElementTraitName: $UpdateElementTraitName:ident
            ElementUpdaterType: $ElementUpdaterType:ident
        }
        for_attributes {
            //TraitDefinitionTokens: ($($TraitDefinitionTokens:tt)+)
            TraitName: $AttributeTraitName:ident
        }
        ambiguous_attributes: $(
            $attribute_type:ident $method_name:ident $(:a:$attribute_name:literal)? $(:e:$element_name:literal)?
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
            UpdateElementTraitName: $UpdateElementTraitName
            ElementUpdaterType: $ElementUpdaterType
            elements: $($method_name $($element_name)?)+
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
