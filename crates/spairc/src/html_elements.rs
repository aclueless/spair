#![allow(non_camel_case_types)]

pub trait ElementMethods: Sized {
    fn id(self, _value: &str) -> Self {
        self
    }

    fn class(self, _value: &str) -> Self {
        self
    }

    fn classes(self, _value: &str) -> Self {
        self
    }
}

macro_rules! make_elements {
    ($($html_element:ident)+) => {
        $(
            pub struct $html_element;
            impl ElementMethods for $html_element {}
        )+
    };
}

make_elements! {
    div span button input
}
