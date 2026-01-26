use std::ops::Not;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{ItemImpl, Result, parse_macro_input};

mod component;
mod dom;
mod view;

#[proc_macro_attribute]
pub fn impl_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match component::Component::from_item_impl(item_impl) {
        Ok(component) => component.generate(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
        TokenStream::new()
    } else {
        output.into()
    }
}

#[proc_macro_attribute]
pub fn create_view(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl: ItemImpl = parse_macro_input!(input);

    let output = match view::View::from_item_impl(item_impl) {
        Ok(view) => view.generate(),
        Err(error) => error.to_compile_error(),
    };
    if args.is_empty().not() {
        println!("{output}");
        TokenStream::new()
    } else {
        output.into()
    }
}

#[derive(Default)]
struct MultiErrors {
    error: Option<syn::Error>,
}

impl MultiErrors {
    fn error_at(&mut self, span: Span, message: &str) {
        let error = syn::Error::new(span, message);
        self.combine(error);
    }

    fn error_at2(&mut self, span: Span, message: &str, message2: &str) {
        let error = syn::Error::new(span, format!("{message} {message2}"));
        self.combine(error);
    }

    fn combine(&mut self, error: syn::Error) {
        if let Some(current_error) = self.error.as_mut() {
            current_error.combine(error);
        } else {
            self.error = Some(error);
        }
    }

    fn report_error(&mut self) -> Result<()> {
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        Ok(())
    }

    fn with_last_error_at<T>(
        &mut self,
        span: Span,
        message: &str,
    ) -> std::result::Result<T, syn::Error> {
        let error = syn::Error::new(span, message);
        if let Some(mut current_error) = self.error.take() {
            current_error.combine(error);
            Err(current_error)
        } else {
            Err(error)
        }
    }
}
