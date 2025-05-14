use std::ops::Not;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, ItemImpl};

mod component_for;
mod element;
mod new_view;

#[proc_macro_attribute]
pub fn new_view(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match new_view::View::with_item_impl(item_impl, false) {
        Ok(view) => view.output(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[proc_macro_attribute]
pub fn component_for(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match new_view::View::with_item_impl(item_impl, false)
        .map(component_for::Component::from_view)
    {
        Ok(component) => component.output(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[derive(Default)]
struct MultiErrors {
    error: Option<syn::Error>,
}

impl MultiErrors {
    fn add(&mut self, span: Span, message: &str) {
        let error = syn::Error::new(span, message);
        self.add_error(error);
    }
    fn add_error(&mut self, error: syn::Error) {
        if let Some(current_error) = self.error.as_mut() {
            current_error.combine(error);
        } else {
            self.error = Some(error);
        }
    }

    fn report_error(self) -> syn::Result<()> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(())
    }
}

mod item_counter {
    use proc_macro2::Span;

    use syn::Ident;

    pub struct ItemCounter {
        namespace: String,
        counter: u32,
    }

    impl ItemCounter {
        pub fn new(namespace: String) -> Self {
            Self {
                namespace,
                counter: 1,
            }
        }

        fn new_ident(&mut self, name: &str) -> Ident {
            let mut name = name.to_string();
            name.push_str(&self.counter.to_string());
            self.counter += 1;
            Ident::new(&name, Span::call_site())
        }

        pub fn new_ident_text(&mut self) -> Ident {
            self.new_ident("_text_")
        }

        pub fn new_ident_element(&mut self, element_name: &str) -> Ident {
            self.new_ident(&format!("_el_{element_name}_"))
        }

        pub fn new_ident_view(&mut self) -> Ident {
            self.new_ident("_view_")
        }

        pub fn new_ident_match(&mut self) -> Ident {
            self.new_ident("_match_")
        }

        pub fn new_ident_ilist(&mut self) -> Ident {
            self.new_ident("_ilist_")
        }

        pub fn new_ident_marker(&mut self, prefix: &str) -> Ident {
            self.new_ident(&format!("_{prefix}_marker_"))
        }

        fn with_namespace(&mut self, prefix: &str) -> Ident {
            let mut name = self.namespace.clone();
            name.push_str(prefix);
            name.push_str(&self.counter.to_string());
            self.counter += 1;
            Ident::new(&name, Span::call_site())
        }

        pub fn new_type_name_match_view_state(&mut self) -> Ident {
            self.with_namespace("MatchVS")
        }

        pub fn new_type_name_match_arm_variant(&mut self) -> Ident {
            self.with_namespace("Arm")
        }

        pub fn new_type_name_inlined_list(&mut self) -> Ident {
            self.with_namespace("InlinedList")
        }
    }
}
