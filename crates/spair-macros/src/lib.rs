use std::ops::Not;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_macro_input, Ident, ItemImpl};

mod component_for;
mod element;
mod keyed_list_item_for;
mod list_item_for;
mod new_view;

#[proc_macro_attribute]
pub fn new_view(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match new_view::View::with_item_impl(item_impl) {
        Ok(view) => view.into_token_stream(),
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

    let output =
        match new_view::View::with_item_impl(item_impl).map(component_for::Component::from_view) {
            Ok(component) => component.output(),
            Err(error) => error.to_compile_error(),
        };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[proc_macro_attribute]
pub fn keyed_list_item_for(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match keyed_list_item_for::KeyedListItemView::with_item_impl(item_impl) {
        Ok(keyed_item_view) => keyed_item_view.output(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
    }
    output.into()
}

#[proc_macro_attribute]
pub fn list_item_for(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match list_item_for::ListItemView::with_item_impl(item_impl) {
        Ok(list_item_view) => list_item_view.output(),
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

struct ItemCounter {
    namespace: String,
    counter: u32,
}
impl ItemCounter {
    fn new(namespace: String) -> Self {
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

    fn new_ident_text(&mut self) -> Ident {
        self.new_ident("_text_")
    }

    fn new_ident_element(&mut self) -> Ident {
        self.new_ident("_element_")
    }

    fn new_ident_view(&mut self) -> Ident {
        self.new_ident("_view_")
    }

    fn new_match_view_state(&mut self) -> Ident {
        let mut name = self.namespace.clone();
        name.push_str("MatchViewState");
        name.push_str(&self.counter.to_string());
        self.counter += 1;
        Ident::new(&name, Span::call_site())
    }
}
