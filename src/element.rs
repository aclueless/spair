use wasm_bindgen::JsCast;

pub struct EventTarget(pub(crate) Option<web_sys::EventTarget>);
pub struct InputElement(pub(crate) web_sys::HtmlInputElement);
pub struct SelectElement(pub(crate) web_sys::HtmlSelectElement);
pub struct FormElement(pub(crate) web_sys::HtmlFormElement);

duplicate::duplicate! {
    [
        TypeName            into_name;
        [HtmlInputElement]  [into_input_element];
        [HtmlSelectElement]  [into_select_element];
        [HtmlFormElement]   [into_form_element];
        [HtmlTextAreaElement] [into_text_area_element];
    ]
    pub struct TypeName(pub(crate) web_sys::TypeName);
    impl TypeName {
        pub fn into_inner(self) -> web_sys::TypeName {
            self.0
        }
    }
    impl EventTarget {
        pub fn into_name(self) -> Option<TypeName> {
            self.into_ws_element().map(TypeName)
        }
    }
}

impl EventTarget {
    pub fn into_ws_element<T: JsCast>(self) -> Option<T> {
        self.0.and_then(|v| v.dyn_into().ok())
    }
}
