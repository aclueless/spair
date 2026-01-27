use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident};

use super::{LastNode, stage::Stage};

pub struct CompRef {
    keyword: Ident,
    comp_ref: Expr,
    stage: Stage,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

impl std::fmt::Debug for CompRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.keyword)
    }
}

impl CompRef {
    pub fn new(
        keyword: Ident,
        comp_ref: Expr,
        stage_picker: &super::stage::StagePicker,
        item_counter: &mut super::ItemCounter,
    ) -> Self {
        let stage = stage_picker.stage_of(&comp_ref);
        Self {
            keyword,
            comp_ref,
            stage,
            spair_ident: item_counter.new_ident("_comp_ref_"),
            spair_ident_marker: item_counter.new_ident_marker("_comp_ref_"),
        }
    }

    pub fn first_span(&self) -> proc_macro2::Span {
        self.keyword.span()
    }

    pub fn generate_view_state_struct_fields(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {pub #ident: ::spair::CompNode,}
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        html_string.push_str("<!--comp-ref-->")
    }

    pub fn generate_fn_create(&self, last_node: &LastNode) -> TokenStream {
        let parent = &last_node.parent;
        let ident = &self.spair_ident;
        let marker = &self.spair_ident_marker;
        let comp_ref = &self.comp_ref;
        let get_marker = last_node.get_ws_node(&self.spair_ident_marker);

        quote! {
            #get_marker
            let #ident = #comp_ref.create_comp_node(&#parent, #marker.get_ws_node_ref().clone());
            let #marker = &#ident.comp_marker;
        }
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident_marker
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {
            #ident,
        }
    }

    pub fn generate_fn_update(&self, view_state: &Ident, parent: &Ident) -> TokenStream {
        if self.stage != Stage::Update {
            return TokenStream::new();
        }
        let ident = &self.spair_ident;
        let comp_ref = &self.comp_ref;
        quote! {
            #comp_ref.update_comp_node(&#view_state.#parent, &mut #view_state.#ident);
        }
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {
            #parent.remove_child(&self.#ident.root_element);
        }
    }
}
