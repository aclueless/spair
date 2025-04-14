use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{
    spanned::Spanned,
    token::{Brace, If, Match as SynMatch},
    Arm, Expr, ExprMatch, Ident, Local, Pat, Result, Stmt,
};

use crate::ItemCounter;

use super::Element;

pub struct Match {
    pub match_keyword: SynMatch,
    pub expr: Expr,
    pub brace_token: Brace,
    pub arms: Vec<MatchArm>,
    pub parent_has_only_one_child: bool,

    pub match_view_state_enum_type_name: Ident,
    pub match_view_state_struct_type_name: Ident,
    pub spair_ident: Ident,
    pub spair_ident_marker: Ident,
}

impl Match {
    pub(crate) fn with_expr_match(
        expr_match: syn::ExprMatch,
        item_counter: &mut ItemCounter,
    ) -> Result<Self> {
        let ExprMatch {
            match_token,
            expr,
            arms,
            brace_token,
            ..
        } = expr_match;
        let mut m = Self {
            match_keyword: match_token,
            expr: *expr,
            brace_token,
            parent_has_only_one_child: false,
            match_view_state_enum_type_name: item_counter.new_match_view_state(),
            match_view_state_struct_type_name: item_counter.new_match_view_state(),
            spair_ident: item_counter.new_ident("_match"),
            spair_ident_marker: item_counter.new_ident("_match_marker"),
            arms: Vec::new(),
        };
        for arm in arms {
            m.arms.push(MatchArm::with_arm(arm, item_counter)?);
        }
        Ok(m)
    }

    pub(crate) fn check_html_multi_errors(&self, errors: &mut crate::MultiErrors) {
        for arm in self.arms.iter() {
            if let Some(element) = arm.element.as_ref() {
                element.check_html_multi_errors(errors);
            }
        }
    }

    pub(crate) fn prepare_items_for_generating_code(&mut self, parent_has_only_one_child: bool) {
        self.parent_has_only_one_child = parent_has_only_one_child;
        for arm in self.arms.iter_mut() {
            if let Some(element) = arm.element.as_mut() {
                element.prepare_items_for_generating_code(false);
            }
        }
    }

    pub(crate) fn generate_view_state_struct_fields(&self) -> TokenStream {
        let match_view_state_type_name = &self.match_view_state_struct_type_name;
        let spair_ident = &self.spair_ident;
        quote! {#spair_ident: #match_view_state_type_name,}
    }

    pub(crate) fn generate_match_view_state_types_n_struct_fields(
        &self,
        inner_types: &mut Vec<TokenStream>,
    ) -> TokenStream {
        inner_types.push(self.generate_match_view_state_types());
        self.generate_view_state_struct_fields()
    }

    pub(crate) fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        let spair_ident = &self.spair_ident;
        quote! {#spair_ident,}
    }

    pub fn generate_match_view_state_types(&self) -> TokenStream {
        let match_view_state_enum_type_name = &self.match_view_state_enum_type_name;
        let match_view_state_struct_type_name = &self.match_view_state_struct_type_name;
        let variants: TokenStream = self
            .arms
            .iter()
            .map(|arm| {
                let variant_name = &arm.match_arm_variant_name;
                let type_name = &arm.match_arm_struct_type_name;
                quote! {#variant_name(#type_name),}
            })
            .collect();
        let get_root_element: TokenStream = self
            .arms
            .iter()
            .map(|arm| {
                let variant_name = &arm.match_arm_variant_name;
                quote! {#match_view_state_enum_type_name::#variant_name(value) => value.root_element(),}
            })
            .collect();
        let match_view_state_types = quote! {
            enum #match_view_state_enum_type_name {
                NotRenderYet,
                #variants
            }
            impl #match_view_state_enum_type_name {
                fn root_element(&self) -> Option<&::spair::WsElement> {
                    match self {
                        #get_root_element
                        #match_view_state_enum_type_name::NotRenderYet => None,
                    }
                }
            }
            struct #match_view_state_struct_type_name {
                match_arm_view_state: #match_view_state_enum_type_name,
                marker: Option<::spair::WsNode>,
            }
        };
        let match_arm_types: TokenStream = self
            .arms
            .iter()
            .map(|v| v.generate_match_view_state_types())
            .collect();
        quote! {
            #match_view_state_types
            #match_arm_types
        }
    }

    pub(crate) fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let ident = &self.spair_ident;
        let match_view_state_enum_type_name = &self.match_view_state_enum_type_name;
        let match_view_state_struct_type_name = &self.match_view_state_struct_type_name;
        let marker = &self.spair_ident_marker;
        let get_marker = if self.parent_has_only_one_child {
            quote! {let #marker = None;}
        } else {
            match previous {
                Some(previous) => {
                    quote! {let #marker = Some(#previous.ws_node_ref().next_sibling_ws_node());}
                }
                None => quote! {let #marker = Some(#parent.ws_node_ref().first_ws_node());},
            }
        };
        quote! {
            #get_marker
            let #ident = #match_view_state_struct_type_name {
                match_arm_view_state: #match_view_state_enum_type_name::NotRenderYet,
                marker: #marker,
            };
        }
    }

    pub(crate) fn generate_code_for_update_view_fn_as_child_node(
        &self,
        parent: &Ident,
        view_state: &Ident,
    ) -> TokenStream {
        let match_keyword = &self.match_keyword;
        let expr = &self.expr;
        let match_arm_code: TokenStream = self
            .arms
            .iter()
            .map(|v| v.generate_code_for_update_view_fn(self, parent, view_state))
            .collect();

        // quote! {
        //     #match_keyword #expr {
        //         #match_arm_code
        //     }
        // }

        let mut out = quote! {#match_keyword #expr};
        self.brace_token
            .surround(&mut out, |inner| inner.append_all([match_arm_code]));
        out
    }
}

pub struct MatchArm {
    pub pat: Pat,
    pub guard: Option<(If, Box<Expr>)>,

    pub locals: Vec<Local>,
    pub element: Option<Element>,

    pub match_arm_variant_name: Ident,
    pub match_arm_struct_type_name: Ident,
}

impl MatchArm {
    fn with_arm(arm: Arm, item_counter: &mut ItemCounter) -> Result<Self> {
        let Arm {
            pat, guard, body, ..
        } = arm;

        let match_arm_type_name = item_counter.new_match_view_state();
        let match_arm_variant_name = item_counter.new_ident("V");

        let mut locals = Vec::new();
        let mut first_expr = None;

        let message = "Only one HTML construction expression allow in a match arm.";
        match *body {
            Expr::Block(expr_block) => {
                let mut stmts = expr_block.block.stmts.into_iter();
                while let Some(stmt) = stmts.next() {
                    match stmt {
                        Stmt::Local(local) => locals.push(local),
                        Stmt::Expr(expr, _) => {
                            first_expr = Some(expr);
                            if let Some(stmt) = stmts.next() {
                                return Err(syn::Error::new(stmt.span(), message));
                            }
                            break;
                        }
                        other => {
                            return Err(syn::Error::new(
                                other.span(),
                                "Expected a let-statement or an HTML construction expression",
                            ));
                        }
                    }
                }
            }
            expr => first_expr = Some(expr),
        };
        let mut match_arm = Self {
            pat,
            guard,
            locals,
            element: None,
            match_arm_struct_type_name: match_arm_type_name,
            match_arm_variant_name,
        };
        let Some(first_expr) = first_expr else {
            return Ok(match_arm);
        };
        let mut elements = Element::with_expr(first_expr, item_counter, None)?;
        match elements.first_mut() {
            Some(Element::Html(html)) => html.root_element = true,
            Some(Element::View(_)) => {}
            Some(other) => {
                return Err(syn::Error::new(
                    other.name_or_text_expr_span(),
                    "Only an HTML element or a view are allowed in a match-arm HTML expression",
                ))
            }
            None => {}
        }
        match_arm.element = elements.pop();
        Ok(match_arm)
    }

    fn generate_match_view_state_types(&self) -> TokenStream {
        let struct_name = &self.match_arm_struct_type_name;
        if let Some(element) = self.element.as_ref() {
            let mut inner_types: Vec<TokenStream> = Vec::new();
            let fields = element.generate_match_view_state_types_n_struct_fields(&mut inner_types);

            let view_state = Ident::new("_local_self_view_state_", Span::call_site());
            let get_root_element = element.generate_get_root_ws_element_4_match_arm(&view_state);
            let remove_from_parent =
                element.generate_remove_child_b4_changing_to_other_match_arm(&view_state);
            quote! {
                #(#inner_types)*
                struct #struct_name {
                    #fields
                }
                impl #struct_name {
                    fn root_element(&self) -> Option<&::spair::WsElement> {
                        let #view_state = self;
                        #get_root_element
                    }
                    fn remove_from_parent(&self, parent: &::spair::WsElement) {
                        let #view_state = self;
                        #remove_from_parent
                    }
                }
            }
        } else {
            quote! {
                struct #struct_name;
                impl #struct_name {
                    fn root_element(&self) -> Option<&::spair::WsElement> {
                        None
                    }
                    fn remove_from_parent(&self, _parent: &::spair::WsElement) {}
                }
            }
        }
    }

    fn generate_code_for_update_view_fn(
        &self,
        m: &Match,
        parent: &Ident,
        view_state: &Ident,
    ) -> TokenStream {
        let pat = &self.pat;
        let guard = match self.guard.as_ref() {
            Some((kw, expr)) => quote! {#kw #expr},
            None => quote! {},
        };
        let locals = &self.locals;
        let match_view_state = &m.spair_ident;
        let enum_type_name = &m.match_view_state_enum_type_name;
        let check_current_view_state: TokenStream = m
            .arms
            .iter()
            .map(|arm| {
                let arm_variant_value = Ident::new(
                    "_arm_variant_value_for_removing_from_parent",
                    Span::call_site(),
                );
                let code = if arm.match_arm_variant_name == self.match_arm_variant_name {
                    quote! { false }
                } else {
                    quote! {
                        #arm_variant_value.remove_from_parent(&#view_state.#parent);
                        true
                    }
                };
                let arm_variant = &arm.match_arm_variant_name;
                quote! {
                    #enum_type_name::#arm_variant(#arm_variant_value) => {
                        #code
                    }
                }
            })
            .collect();
        let create_new_view_state = self.generate_create_new_view_state(m, parent, view_state);
        let variant = &self.match_arm_variant_name;
        let match_arm_view_state = Ident::new("_match_arm_view_state", Span::call_site());
        let update_match_arm_view_state = self.generate_update_view(&match_arm_view_state);
        quote! {
            #pat #guard => {
                #(#locals)*
                let _must_create_new_view_state_ = match &#view_state.#match_view_state.match_arm_view_state {
                    #check_current_view_state
                    _ => {true}
                };
                if _must_create_new_view_state_ {
                    #create_new_view_state
                }
                if let #enum_type_name::#variant(#match_arm_view_state) = &mut #view_state.#match_view_state.match_arm_view_state {
                    #update_match_arm_view_state
                }
            }
        }
    }

    fn generate_create_new_view_state(
        &self,
        m: &Match,
        parent: &Ident,
        view_state: &Ident,
    ) -> TokenStream {
        let match_view_state = &m.spair_ident;
        let match_enum_type_name = &m.match_view_state_enum_type_name;
        let arm_variant = &self.match_arm_variant_name;
        let view_state_struct_name = &self.match_arm_struct_type_name;
        match self.element.as_ref() {
            None => {
                quote! {#view_state.#match_view_state.match_arm_view_state = #match_enum_type_name::#arm_variant(#view_state_struct_name{});}
            }
            Some(element) => match element {
                Element::Text(_) => quote! {},
                Element::Html(html_element) => {
                    let create_view = html_element.generate_code_for_create_view_fn();
                    let construct_view_state_instance = html_element
                        .generate_view_state_instance_construction(view_state_struct_name);
                    let root_element = &html_element.meta.spair_ident;
                    let match_arm_view_state =
                        Ident::new("_match_arm_view_state", Span::call_site());
                    quote! {
                        #create_view
                        let #match_arm_view_state = #construct_view_state_instance;
                        #view_state.#parent.insert_new_node_before_a_node(&#match_arm_view_state.#root_element,#view_state.#match_view_state.marker.as_deref());
                        #view_state.#match_view_state.match_arm_view_state = #match_enum_type_name::#arm_variant(#match_arm_view_state);
                    }
                }
                Element::View(view) => {
                    let view_ident = &view.spair_ident;
                    let view_name = &view.name;
                    let create_view_args = &view.create_view_args;
                    let match_arm_view_state =
                        Ident::new("_match_arm_view_state", Span::call_site());
                    quote! {
                        let #match_arm_view_state = #view_name::create_view(#create_view_args);
                        #view_state.#parent.insert_new_node_before_a_node(#match_arm_view_state.root_element(), #view_state.#match_view_state.marker.as_deref());
                        let #match_arm_view_state = #view_state_struct_name{#view_ident: #match_arm_view_state};
                        #view_state.#match_view_state.match_arm_view_state = #match_enum_type_name::#arm_variant(#match_arm_view_state);
                    }
                }
                Element::List(_keyed_list) => todo!(),
                Element::Match(_) => todo!(),
            },
        }
    }

    fn generate_update_view(&self, view_state: &Ident) -> TokenStream {
        match self.element.as_ref() {
            None => {
                quote! {
                    //No element
                }
            }
            Some(mvs) => match mvs {
                Element::Text(_) => quote! {},
                Element::Html(html_element) => {
                    html_element.generate_code_for_update_view_fn_as_child_node(view_state)
                }
                Element::View(view) => {
                    let ident = &view.spair_ident;
                    if let Some(update_view_args) = view.update_view_args.as_ref() {
                        quote! {
                            #view_state.#ident.update_view(#update_view_args);
                        }
                    } else {
                        quote! {}
                    }
                }
                Element::List(_keyed_list) => todo!(),
                Element::Match(_) => todo!(),
            },
        }
    }
}
