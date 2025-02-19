use std::ops::Not;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, Ident, ItemImpl};

mod component;
mod element;
mod keyed_item_view;
mod view;

#[proc_macro_attribute]
pub fn view(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match view::View::with_item_impl(item_impl) {
        Ok(view) => view.into_token_stream(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match view::View::with_item_impl(item_impl).map(component::Component::from_view) {
        Ok(component) => component.output(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[proc_macro_attribute]
pub fn keyed_item_view(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match keyed_item_view::KeyedItemView::with_item_impl(item_impl) {
        Ok(keyed_item_view) => keyed_item_view.output(),
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

struct ItemCounter(u32);
impl ItemCounter {
    fn new() -> Self {
        Self(1)
    }

    fn new_ident(&mut self, name: &str) -> Ident {
        let mut name = name.to_string();
        name.push_str(&self.0.to_string());
        self.0 += 1;
        Ident::new(&name, Span::call_site())
    }

    fn new_ident_text(&mut self) -> Ident {
        self.new_ident("_text_")
    }

    fn new_ident_element(&mut self) -> Ident {
        self.new_ident("_element_")
    }

    fn new_ident_view(&mut self) -> Ident {
        self.new_ident("_view_")
    }
}
