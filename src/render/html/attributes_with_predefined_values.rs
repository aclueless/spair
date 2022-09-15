use super::{AttributesOnly, HtmlElementRender, StaticAttributes, StaticAttributesOnly};
use crate::component::Component;

#[cfg(test)]
use crate::render::html::TestHtmlMethods;

make_trait_for_attributes_with_predefined_values! {
TestStructs: (TestHtmlMethods)
TraitName: HamsWithPredefinedValues
{
    AutoCapitalizeAttributeValue
    AutoCapitalize {
        Off => "off",
        On => "on",
        Sentences => "sentences",
        Words => "words",
        Characters => "characters",
    }
    {}
}
{
    AutoCompleteAttributeValue
    AutoComplete {
        On => "on",
        Off => "off",
    }
    { auto_complete "autocomplete" }
}
{
    ButtonTypeAttributeValue
    ButtonType {
        Button => "button",
        Submit => "submit",
        Reset => "reset",
    }
    {}
}
{
    CrossOriginAttributeValue
    CrossOrigin {
        Anonymous => "anonymous",
        UseCredentials => "use-credentials",
    }
    { cross_origin "crossorigin" }
}
{
    DecodingAttributeValue
    Decoding {
        Sync => "sync",
        Async => "async",
        Auto => "auto",
    }
    { decoding }
}
{
    DirAttributeValue
    Dir {
        LeftToRight => "ltr",
        RightToLeft => "rtl",
        Auto => "auto",
    }
    {}
}
{
    EncTypeAttributeValue
    EncType {
        ComponentXWwwFormUrlEncoded => "Component/x-www-form-urlencoded",
        MultiPartFormData => "multipart/form-data",
        TextPlain => "text/plain",
    }
    {
        enc_type "enctype"
        form_enc_type "formenctype"
    }
}
{
    FormMethodAttributeValue
    FormMethod {
        Post => "post",
        Get => "get",
    }
    {
        form_method "formmethod"
        method
    }
}
{
    InputModeAttributeValue
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
    {}
}
{
    InputTypeAttributeValue
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
    { r#type "type" }
}
{
    OlTypeAttributeValue
    OlType {
        Number => "1",
        LowerCase => "a",
        UpperCase => "C",
        LowerCaseRoman => "i",
        UpperCaseRoman => "I",
    }
    {}
}
{
    PreLoadAttributeValue
    PreLoad {
        None => "none",
        MetaData => "metadata",
        Auto => "auto",
    }
    { pre_load "preload" }
}
{
    ReferrerPolicyAttributeValue
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
    { referrer_policy "referrerpolicy" }
}
{
    SandboxAttributeValue
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
    {}
}
{
    SpellCheckAttributeValue
    SpellCheck {
        True => "true",
        False => "false",
        Default => "default",
    }
    {}
}
{
    ThScopeAttributeValue
    ThScope {
        Row => "row",
        Col => "col",
        RowGroup => "rowgroup",
        ColGroup => "colgroup",
        Auto => "auto",
    }
    { scope }
}
{
    TargetAttributeValue
    Target {
        _Self => "_self",
        _Blank => "_blank",
        _Parent => "_parent",
        _Top => "_top",
    }
    {
        form_target "formtarget"
        target
    }
}
{
    TrackKindAttributeValue
    TrackKind {
        Subtitles => "subtitles",
        Captions => "captions",
        Descriptions => "descriptions",
        Chapters => "chapters",
        Metadata => "metadata",
    }
    { kind }
}
{
    WrapAttributeValue
    Wrap {
        Hard => "hard",
        Soft => "soft",
    }
    { wrap }
}
}

impl<'er, C: Component> HamsWithPredefinedValues<C> for HtmlElementRender<'er, C> {}
impl<'er, C: Component> HamsWithPredefinedValues<C> for StaticAttributes<'er, C> {}
impl<'er, C: Component> HamsWithPredefinedValues<C> for AttributesOnly<'er, C> {}
impl<'er, C: Component> HamsWithPredefinedValues<C> for StaticAttributesOnly<'er, C> {}
