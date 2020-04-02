mod sealed {
    pub trait AsStr {}
}

pub trait AsStr: sealed::AsStr {
    fn as_str(&self) -> &str;
}

macro_rules! create_as_str_enums {
    ($(
        $(#[$enum_meta_content:meta])*
        $EnumTypeName:ident { $(
            $(#[$variant_meta_content:meta])*
            $VariantName:ident => $str_value:literal,
        )+}
    )+) => {
        $(
            $(#[$enum_meta_content])*
            pub enum $EnumTypeName {
                $(
                    $(#[$variant_meta_content])*
                    $VariantName,
                )+
            }

            impl sealed::AsStr for $EnumTypeName {}

            impl AsStr for $EnumTypeName {
                fn as_str(&self) -> &str {
                    match self {
                        $(
                            #[allow(deprecated)]
                            $EnumTypeName::$VariantName => $str_value,
                        )+
                    }
                }
            }
        )+
    };
}

create_as_str_enums! {
    InputType {
        Button => "button",
        CheckBox => "checkbox",
        Color => "color",
        Date => "date",
        DateTimeLocal => "datetime-local",
        Email => "email",
        FileUpload => "file",
        Hidden => "hidden",
        ImageButton => "image",
        Month => "month",
        Number => "number",
        Password => "password",
        RadioButton => "radio",
        Range => "range",
        ResetButton => "reset",
        Search => "search",
        SubmitButton => "submit",
        Telephone => "tel",
        Text => "text",
        Time => "time",
        Url => "url",
        Week => "week",
    }
}
