use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Ident, spanned::Spanned};

use crate::dom::stage::Stage;

use super::{LastNode, stage::StagePicker};

pub struct Text {
    text_ident: Option<Ident>,
    stage: Stage,
    value: Expr,

    next_node_is_a_text: bool,
    spair_ident: Ident,
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "text {}",
            self.text_ident
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default()
        )
    }
}

impl Text {
    pub fn with_expr_lit(
        text_ident: Option<Ident>,
        expr_lit: syn::PatLit,
        text_value: String,
        item_counter: &mut super::ItemCounter,
    ) -> Self {
        Text {
            text_ident,
            stage: Stage::HtmlString(text_value),
            value: Expr::Lit(expr_lit),

            next_node_is_a_text: false,
            spair_ident: item_counter.new_ident_text(),
        }
    }

    pub fn with_non_expr_lit(
        text_ident: Option<Ident>,
        expr_as_a_text_node: Expr,
        item_counter: &mut super::ItemCounter,
        stage_picker: &StagePicker,
    ) -> Self {
        Text {
            text_ident,
            stage: stage_picker.stage_of(&expr_as_a_text_node),
            value: expr_as_a_text_node,

            next_node_is_a_text: false,
            spair_ident: item_counter.new_ident_text(),
        }
    }

    pub fn first_span(&self) -> Span {
        match &self.text_ident {
            Some(text_ident) => text_ident.span(),
            None => self.value.span(),
        }
    }

    pub fn generate_view_state_struct_fields(&self) -> TokenStream {
        if self.stage == Stage::Update {
            let ident = &self.spair_ident;
            quote! {pub #ident: ::spair::Text,}
        } else {
            quote! {}
        }
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        if let Stage::HtmlString(text_value) = &self.stage {
            html_string.push_str(text_value);
        } else {
            html_string.push_str("\u{202F}");
        }
    }

    pub fn generate_fn_create(&self, last_node: &LastNode) -> TokenStream {
        let text_node = &self.spair_ident;
        let offset = match &self.stage {
            Stage::HtmlString(s) => s.chars().count() as u32,
            Stage::Creation => 1,
            Stage::Update => 1,
        };
        let text_value = &self.value;
        let get_text_node = last_node.get_text(&self.spair_ident);
        let get_ws_text_node = last_node.get_ws_text(&self.spair_ident);
        let offset = if self.next_node_is_a_text {
            quote! {
                #text_node.split_text(#offset);
            }
        } else {
            quote! {}
        };
        match self.stage {
            Stage::Creation => {
                quote! {
                    #get_ws_text_node
                    #offset
                    #text_node.set_text(#text_value);
                }
            }
            Stage::Update => {
                quote! {
                    #get_text_node
                    #offset
                }
            }
            Stage::HtmlString(_) => {
                quote! {
                    #get_ws_text_node
                    #offset
                }
            }
        }
    }

    pub fn set_next_is_a_text(&mut self) {
        self.next_node_is_a_text = true;
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        if self.stage == Stage::Update {
            let ident = &self.spair_ident;
            quote! {#ident,}
        } else {
            quote! {}
        }
    }

    pub fn generate_fn_update(&self, view_state: &Ident) -> TokenStream {
        let text_node = &self.spair_ident;
        let text_value = &self.value;
        if let Stage::Update = self.stage {
            quote! {
                #view_state.#text_node.update(#text_value);
            }
        } else {
            quote! {}
        }
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        let text_node = &self.spair_ident;
        quote! {
            #parent.remove_child(&self.#text_node);
        }
    }
}
