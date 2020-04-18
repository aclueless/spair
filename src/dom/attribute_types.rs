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
    AutoCapitalize {
        Off => "off",
        On => "on",
        Sentences => "sentences",
        Words => "words",
        Characters => "characters",
    }
    AutoComplete {
        On => "on",
        Off => "off",
    }
    ButtonType {
        Button => "button",
        Submit => "submit",
        Reset => "reset",
    }
    CrossOrigin {
        Anonymous => "anonymous",
        UseCredentials => "use-credentials",
    }
    Decoding {
        Sync => "sync",
        Async => "async",
        Auto => "auto",
    }
    Dir {
        LeftToRight => "ltr",
        RightToLeft => "rtl",
        Auto => "auto",
    }
    EncType {
        ComponentXWwwFormUrlEncoded => "Component/x-www-form-urlencoded",
        MultiPartFormData => "multipart/form-data",
        TextPlain => "text/plain",
    }
    FormMethod {
        Post => "post",
        Get => "get",
    }
    InputMode {
        None => "none",
        Text => "text",
        Decimal => "decimal",
        Numeric => "numeric",
        Tel => "tel",
        Search => "search",
        Email => "email",
        Url => "url",
    }
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
    OlType {
        Number => "1",
        LowerCase => "a",
        UpperCase => "C",
        LowerCaseRoman => "i",
        UpperCaseRoman => "I",
    }
    PreLoad {
        None => "none",
        MetaData => "metadata",
        Auto => "auto",
    }
    ReferrerPolicy {
        NoReferrer => "no-referrer",
        NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
        Origin => "origin",
        OriginWhenCrossOrigin => "origin-when-cross-origin",
        SameOrigin => "same-origin",
        StrictOrigin => "strict-origin",
        StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
        UnsafeUrl => "unsafe-url",
    }
    Sandbox {
        AllowForms => "allow-forms",
        AllowModals => "allow-modals",
        AllowOrientationLock => "allow-orientation-lock",
        AllowPointerLock => "allow-pointer-lock",
        AllowPopups => "allow-popups",
        AllowPopupsToEscapeSandbox => "allow-popups-to-escape-sandbox",
        AllowPresentation => "allow-presentation",
        AllowSameOrigin => "allow-same-origin",
        AllowScripts => "allow-scripts",
        // Should we have feature = "experimental" for things like this?
        // allow-storage-access-by-user-activation
        AllowTopNavigation => "allow-top-navigation",
        AllowTopNavigationByUserActivation => "allow-top-navigation-by-user-activation",
        //AllowDownloadsWithoutUserActivation => "allow-downloads-without-user-activation",
    }
    SpellCheck {
        True => "true",
        False => "false",
        Default => "default",
    }
    ThScope {
        Row => "row",
        Col => "col",
        RowGroup => "rowgroup",
        ColGroup => "colgroup",
        Auto => "auto",
    }
    Target {
        _Self => "_self",
        #[deprecated(note = "There is a Security_and_privacy_concerns when `target='_blank'`, please use `.target_blank_with_rel()`. See more at https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a#Security_and_privacy_concerns")]
        _Blank => "_blank",
        _Parent => "_parent",
        _Top => "_top",
    }
    TrackKind {
        Subtitles => "subtitles",
        Captions => "captions",
        Descriptions => "descriptions",
        Chapters => "chapters",
        Metadata => "metadata",
    }
    Wrap {
        Hard => "hard",
        Soft => "soft",
    }
}
