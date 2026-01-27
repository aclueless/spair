use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Expr, Ident,
    punctuated::Punctuated,
    token::{self, Comma},
};

use super::{ItemCounter, LastNode};

pub struct View {
    view_name: Ident,
    create_call: ViewFnCall,
    update_call: Option<ViewFnCall>,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "view {}", self.view_name)
    }
}

pub struct ViewFnCall {
    fn_name: Ident,
    paren_token: token::Paren,
    fn_args: Punctuated<Expr, Comma>,
}

impl ViewFnCall {
    pub fn new(
        fn_name: Ident,
        paren_token: token::Paren,
        fn_args: Punctuated<Expr, Comma>,
    ) -> Self {
        Self {
            fn_name,
            paren_token,
            fn_args,
        }
    }
}

impl View {
    pub fn new(
        view_name: Ident,
        create_call: ViewFnCall,
        update_call: Option<ViewFnCall>,
        item_counter: &mut ItemCounter,
    ) -> Self {
        Self {
            view_name,
            create_call,
            update_call,

            spair_ident: item_counter.new_ident_view(),
            spair_ident_marker: item_counter.new_ident_marker("view"),
        }
    }

    pub fn first_span(&self) -> Span {
        self.view_name.span()
    }

    pub fn generate_view_state_struct_fields(&self, for_match_arm: bool) -> TokenStream {
        let ident = &self.spair_ident;
        let type_name = &self.view_name;
        if for_match_arm {
            quote! {pub #ident: super::#type_name,}
        } else {
            quote! {pub #ident: #type_name,}
        }
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        html_string.push_str("<!--view-->");
    }

    pub fn generate_fn_create(&self, last_node: &LastNode) -> TokenStream {
        let parent = &last_node.parent;
        let view_state = &self.spair_ident;
        let view_name = &self.view_name;
        let view_marker = &self.spair_ident_marker;
        let create_view_fn_name = &self.create_call.fn_name;

        let create_view_fn_args = &self.create_call.fn_args;
        let paren = self.create_call.paren_token;
        let mut ts = quote! {};
        paren.surround(&mut ts, |inner| {
            inner.extend(
                quote! {&#parent, Some(#view_marker.get_ws_node_ref()), #create_view_fn_args},
            );
        });
        let create_view_fn_args = ts;

        let get_marker = last_node.get_ws_node(&self.spair_ident_marker);
        quote! {
            #get_marker
            // let #view_state = #view_name::#create_view_fn_name(&#parent, Some(#view_marker.get_ws_node_ref()), #create_view_fn_args);
            let #view_state = #view_name::#create_view_fn_name #create_view_fn_args;
        }
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident_marker
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {#ident,}
    }

    pub fn generate_fn_update(&self, parent_view_state: &Ident, parent: &Ident) -> TokenStream {
        let this_view_state = &self.spair_ident;
        if let Some(update_call) = self.update_call.as_ref() {
            let update_view_fn_name = &update_call.fn_name;
            let update_view_fn_args = &update_call.fn_args;
            let paren = update_call.paren_token;
            let mut ts = quote! {};
            paren.surround(&mut ts, |inner| {
                inner.extend(quote! {&#parent_view_state.#parent, #update_view_fn_args});
            });
            let update_view_fn_args = ts;
            quote! {
                // #parent_view_state.#this_view_state.#update_view_fn_name(&#parent_view_state.#parent, #update_view_fn_args);
                #parent_view_state.#this_view_state.#update_view_fn_name #update_view_fn_args;
            }
        } else {
            // This will cause a compile error if the view requires an update call but the user
            // forget calling it.
            let update_view_fn_name = Ident::new("update", self.view_name.span());
            quote! {
                #parent_view_state.#this_view_state.#update_view_fn_name();
            }
        }
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        let view_state = &self.spair_ident;
        quote! {
            self.#view_state.remove_from(#parent);
        }
    }

    pub fn has_update_call(&self) -> bool {
        self.update_call.is_some()
    }
}
