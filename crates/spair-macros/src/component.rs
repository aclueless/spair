use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, Signature};

use crate::{element::HtmlElement, view::View};

pub struct Component {
    component_name: Ident,
    item_impl: ItemImpl,
    element: HtmlElement,
}

impl Component {
    pub fn from_view(view: View) -> Self {
        Self {
            component_name: view.view_state_type_name,
            item_impl: view.item_impl,
            element: view.element,
        }
    }

    pub fn output(&self) -> TokenStream {
        let component_view_state_struct_name = Ident::new(
            &format!("{}ViewState", self.component_name),
            Span::call_site(),
        );
        let struct_fields = self.element.generate_view_state_struct_fields();
        let view_state_struct =
            quote! {pub struct #component_view_state_struct_name{#struct_fields}};
        let impl_component = self.generate_impl_component(&component_view_state_struct_name);

        quote! {
            #view_state_struct
            #impl_component
        }
    }

    fn generate_impl_component(&self, view_state_struct_name: &Ident) -> TokenStream {
        let generated_create_view_fn = self.generate_impl_create_view_fn(view_state_struct_name);
        let generated_update_view_fn = self.generate_impl_update_view_fn();

        let impl_token = &self.item_impl.impl_token;
        let self_type = &self.item_impl.self_ty;
        quote! {
            #impl_token Component for #self_type {
                type ViewState = #view_state_struct_name;
                #generated_create_view_fn
                #generated_update_view_fn
            }
        }
    }

    fn generate_impl_create_view_fn(&self, view_state_struct_name: &Ident) -> TokenStream {
        let impl_item = self.item_impl.items.get(0);
        let html_string = self.element.construct_html_string();
        let fn_body = self
            .element
            .generate_code_for_create_view_fn_of_a_component(view_state_struct_name, &html_string);
        let ImplItem::Fn(ImplItemFn { sig, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        quote! {
            #sig -> (WsElement, Self::ViewState) {
                #fn_body
            }
        }
    }

    fn generate_impl_update_view_fn(&self) -> TokenStream {
        let view_state_ident = Ident::new(
            "_spair_component_view_state_for_updating_",
            Span::call_site(),
        );
        let impl_item = self.item_impl.items.get(1);
        let fn_body = self
            .element
            .generate_code_for_update_view_fn(&view_state_ident);
        let ImplItem::Fn(ImplItemFn { sig, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        let Signature {
            fn_token,
            ident,
            inputs,
            ..
        } = sig;
        quote! {
            #fn_token #ident(#view_state_ident: &mut Self::ViewState, #inputs) {
                #fn_body
            }
        }
    }
}
