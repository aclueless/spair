use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, ExprMatch, Ident, spanned::Spanned, token::Brace};

use crate::MultiErrors;

use super::{
    Items, LastNode, SubMod, expr_as_ident,
    stage::{Stage, StagePicker},
};

pub struct Match {
    expr_match: ExprMatch,
    arm_views: Vec<ArmView>,

    stage: Stage,
    parent_has_only_one_child: bool,

    match_enum_name: Ident,
    match_struct_name: Ident,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

impl std::fmt::Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "match {}", self.match_enum_name)
    }
}

struct ArmView {
    args: Vec<Expr>,
    items: Items,

    arm_struct_view_name: Ident,
    match_enum_variant_for_arm: Ident,

    fn_create_ident: Ident,
    fn_update_ident: Ident,
}

impl Match {
    pub fn first_span(&self) -> Span {
        self.expr_match.match_token.span()
    }

    pub fn with_expr_match(
        expr_match: ExprMatch,
        stage_picker: &StagePicker,
        item_counter: &mut super::ItemCounter,
        errors: &mut MultiErrors,
    ) -> Match {
        let stage = stage_picker.stage_of(&expr_match.expr);
        let mut m = Match {
            expr_match,
            arm_views: Vec::new(),

            stage,
            parent_has_only_one_child: false,

            match_enum_name: item_counter.new_match_enum(),
            match_struct_name: item_counter.new_match_struct(),

            spair_ident: item_counter.new_ident_match(),
            spair_ident_marker: item_counter.new_ident_marker("match"),
        };
        let stage_picker = if m.stage == Stage::Update {
            StagePicker::DefaultToUpdate
        } else {
            StagePicker::DefaultToCreation
        };
        for ma in m.expr_match.arms.iter_mut() {
            let mut arm_body = Expr::Block(syn::ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: syn::Block {
                    brace_token: Brace::default(),
                    stmts: Vec::new(),
                },
            });
            std::mem::swap(&mut arm_body, &mut ma.body);
            let arm_view = ArmView::with_arm_body(arm_body, &stage_picker, item_counter, errors);
            m.arm_views.push(arm_view);
        }
        m
    }

    pub fn validate_html(&self, errors: &mut MultiErrors) {
        for av in self.arm_views.iter() {
            av.items.validate_html(errors);
        }
    }

    pub fn prepare_items_for_generating_code(&mut self, parent_has_only_one_child: bool) {
        self.parent_has_only_one_child = parent_has_only_one_child;
        for arm in self.arm_views.iter_mut() {
            arm.items.prepare_items_for_generating_code();
        }
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        if self.parent_has_only_one_child.not() {
            html_string.push_str("<!--match-->")
        }
    }

    pub fn generate_view_state_struct_fields(&self, sub_mod: &SubMod) -> TokenStream {
        let match_struct_name = sub_mod.generate(&self.match_struct_name);
        let spair_ident = &self.spair_ident;
        quote! {pub #spair_ident: #match_struct_name,}
    }

    pub fn generate_view_states_for_matches_and_lists(&self) -> TokenStream {
        let struct_name = &self.match_struct_name;
        let enum_name = &self.match_enum_name;
        let marker = match self.parent_has_only_one_child {
            true => quote! {},
            false => {
                quote! { marker: ::spair::WsNode, }
            }
        };
        let match_state_struct = quote! {
            pub struct #struct_name{
                pub view_state: #enum_name,
                pub #marker
            }

            impl #struct_name {
                pub fn remove_from(&self, parent: &::spair::WsElement) {
                    self.view_state.remove_from(parent);
                }
            }
        };

        let variants: TokenStream = self
            .arm_views
            .iter()
            .map(|arm| {
                let variant_name = &arm.match_enum_variant_for_arm;
                let type_name = &arm.arm_struct_view_name;
                quote! {#variant_name(#type_name),}
            })
            .collect();
        let remove_from_by_arms: TokenStream = self
            .arm_views
            .iter()
            .map(|v| v.generate_remove_from_by_arm(&self.match_enum_name))
            .collect();
        let match_state_enum = quote! {
            pub enum #enum_name{
                NeverRendered,
                #variants
            }

            impl #enum_name {
                pub fn remove_from(&self, parent: &::spair::WsElement) {
                    match self {
                        #enum_name::NeverRendered => {}
                        #remove_from_by_arms
                    }
                }
            }
        };

        let arm_view_states: TokenStream = self
            .arm_views
            .iter()
            .map(|arm| arm.generate_arm_view_states())
            .collect();

        let inner_view_states: TokenStream = self
            .arm_views
            .iter()
            .map(|v| v.items.generate_view_states_for_matches_and_lists())
            .collect();

        quote! {
            #match_state_struct
            #match_state_enum
            #arm_view_states
            #inner_view_states
        }
    }

    pub fn generate_fn_create(&self, sub_mod: &SubMod, last_node: &LastNode) -> TokenStream {
        let parent = &last_node.parent;
        let initial_view_for_match_item =
            self.generate_fn_create_for_initialization(sub_mod, last_node);
        if self.stage == Stage::Update {
            // When this match item is in update mode, all code for creation will
            // be generate in the view/component update function. This way, when
            // the active march-arm changed, the arm-view can be create with the
            // new arm's values
            return initial_view_for_match_item;
        }

        let expr_match = self.generate_fn_create_code(sub_mod, parent);

        let view_state = &self.spair_ident;
        quote! {
            #initial_view_for_match_item
            #view_state.view_state = #expr_match;
        }
    }

    fn generate_fn_create_for_initialization(
        &self,
        sub_mod: &SubMod,
        last_node: &LastNode,
    ) -> TokenStream {
        let match_view_state = &self.spair_ident;
        let match_struct_name = sub_mod.generate(&self.match_struct_name);
        let match_enum_name = sub_mod.generate(&self.match_enum_name);
        let initial_view_for_match_item = if self.parent_has_only_one_child {
            quote! {
                let mut #match_view_state = #match_struct_name {view_state: #match_enum_name::NeverRendered};
            }
        } else {
            let marker = &self.spair_ident_marker;
            let get_marker = last_node.get_ws_node(&self.spair_ident_marker);
            quote! {
                #get_marker
                let mut #match_view_state = #match_struct_name {
                    view_state: #match_enum_name::NeverRendered,
                    marker: #marker,
                };
                let #marker = &#match_view_state.marker;
            }
        };

        #[allow(clippy::let_and_return)]
        initial_view_for_match_item
    }

    fn generate_fn_create_code(&self, sub_mod: &SubMod, parent_of_the_match: &Ident) -> ExprMatch {
        let view_state = &self.spair_ident;
        let mut expr_match = self.expr_match.clone();
        for (arm, arm_view) in expr_match.arms.iter_mut().zip(self.arm_views.iter()) {
            let closure_definition =
                arm_view.generate_fn_create_closure_for_creation(sub_mod, &self.match_enum_name);
            let closure_name = &arm_view.fn_create_ident;
            let arm_code = quote! {{
                #closure_definition
                #closure_name(&#parent_of_the_match, Some(#view_state.marker.get_ws_node_ref()))
            }};
            let arm_code: syn::Block =
                syn::parse(arm_code.into()).expect("fn create match arm code");
            *arm.body = Expr::Block(syn::ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: arm_code,
            });
        }
        expr_match
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident_marker
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        let spair_ident = &self.spair_ident;
        quote! {#spair_ident,}
    }

    pub fn generate_fn_update(
        &self,
        sub_mod: &SubMod,
        view_state: &Ident,
        parent: &Ident,
    ) -> TokenStream {
        if self.stage == Stage::Creation {
            return quote! {};
        }
        let match_view_state = &self.spair_ident;
        let match_view_state = quote! {#view_state.#match_view_state};
        let mut expr_match = self.expr_match.clone();
        for (arm, arm_view) in expr_match.arms.iter_mut().zip(self.arm_views.iter()) {
            let arm_code = arm_view.generate_fn_update(
                &match_view_state,
                sub_mod,
                &self.match_enum_name,
                parent,
            );
            let arm_code: syn::Block =
                syn::parse(arm_code.into()).expect("fn update match arm code");
            *arm.body = Expr::Block(syn::ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: arm_code,
            });
        }
        quote! {
            let #parent = &#view_state.#parent;
            #expr_match
        }
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        let view_state = &self.spair_ident;
        quote! {
            self.#view_state.remove_from(#parent);
        }
    }
}

impl ArmView {
    fn with_arm_body(
        arm_body: Expr,
        stage_picker: &StagePicker,
        item_counter: &mut super::ItemCounter,
        errors: &mut MultiErrors,
    ) -> ArmView {
        let mut arm_view = ArmView {
            args: Vec::new(),
            items: Items::default(),
            arm_struct_view_name: item_counter.new_arm_struct(),
            match_enum_variant_for_arm: item_counter.new_arm_variant(),
            fn_create_ident: item_counter.new_ident("_fn_create"),
            fn_update_ident: item_counter.new_ident("_fn_update"),
        };
        match arm_body {
            Expr::Block(mut expr_block) => {
                let mut new_stage_picker = None;
                if let Some(syn::Stmt::Expr(Expr::Call(expr_call), _)) =
                    expr_block.block.stmts.first()
                    && let Expr::Path(path) = expr_call.func.as_ref()
                    && let Ok(single_ident) = path.path.require_ident()
                    && single_ident == "fn_create"
                {
                    if *stage_picker == StagePicker::DefaultToCreation {
                        errors.error_at(
                            single_ident.span(),
                            "The `match` item is already in creation mode",
                        );
                    }
                    let creation_variables =
                        collect_variable_name_from_expr_call_args(&expr_call.args, errors);
                    arm_view.args.extend(expr_call.args.iter().cloned());
                    new_stage_picker =
                        Some(StagePicker::CheckWithCreationVariables(creation_variables));
                }

                let stage_picker = match new_stage_picker.as_ref() {
                    Some(new_stage_picker) => {
                        let _f = expr_block.block.stmts.remove(0);
                        new_stage_picker
                    }
                    None => stage_picker,
                };
                arm_view.items.collect_from_block(
                    expr_block.block,
                    true,
                    stage_picker,
                    item_counter,
                    errors,
                );
            }
            other_expr => {
                arm_view.items.collect_from_expr(
                    true,
                    other_expr,
                    stage_picker,
                    item_counter,
                    errors,
                );
            }
        }
        arm_view
    }

    fn generate_fn_create_closure_for_creation(
        &self,
        sub_mod: &SubMod,
        match_enum_name: &Ident,
    ) -> TokenStream {
        let let_stmts: TokenStream = self.args.iter().map(|v| quote! {let #v;}).collect();

        let mut html_static_string = String::new();
        self.items.generate_html_string(&mut html_static_string);

        let template_fragment = Ident::new("__spair_view_document_fragment_", Span::call_site());
        let mut last_node = LastNode {
            parent: template_fragment.clone(),
            previous: None,
        };

        let create_elements_code = self.items.generate_fn_create(sub_mod, &mut last_node);

        let match_enum_name = sub_mod.generate(match_enum_name);
        let variant = &self.match_enum_variant_for_arm;
        let view_name = sub_mod.generate(&self.arm_struct_view_name);
        let fields = self.items.generate_fn_create_return_value();
        let view_state_instance_construction = quote! {
            #match_enum_name::#variant(
            #view_name{
                #fields
            })
        };

        let fn_create = &self.fn_create_ident;
        let create_fn_closure_for_creation = quote! {
            let #fn_create = |__spair_parent_of_the_match: &::spair::WsElement, __spair_next_sibling_of_the_match| {
                #let_stmts

                const HTML_STRING: &str = #html_static_string;
                let #template_fragment = ::spair::TemplateElement::new(HTML_STRING).fragment();
                #create_elements_code
                __spair_parent_of_the_match.insert_new_node_before_a_node(&#template_fragment, __spair_next_sibling_of_the_match);
                #view_state_instance_construction
            };
        };
        create_fn_closure_for_creation
    }

    fn generate_fn_update(
        &self,
        match_view_state: &TokenStream,
        sub_mod: &SubMod,
        match_enum_name: &Ident,
        parent: &Ident,
    ) -> TokenStream {
        let fn_create_closure_definition =
            self.generate_fn_create_closure_for_creation(sub_mod, match_enum_name);
        let fn_update_closure_definition =
            self.generate_fn_create_closure_for_update(sub_mod, match_enum_name, parent);
        let fn_create_closure_name = &self.fn_create_ident;
        let fn_update_closure_name = &self.fn_update_ident;

        let match_enum_name = sub_mod.generate(match_enum_name);
        let variant = &self.match_enum_variant_for_arm;
        let arm_code = quote! {{
            #fn_create_closure_definition
            #fn_update_closure_definition
            if !matches!(&#match_view_state.view_state, #match_enum_name::#variant(_)) {
                #match_view_state.view_state.remove_from(#parent);
                #match_view_state.view_state = #fn_create_closure_name(#parent, Some(#match_view_state.marker.get_ws_node_ref()));
            }
            if let #match_enum_name::#variant(__spair_match_arm_local_view_state) = &mut #match_view_state.view_state{
                #fn_update_closure_name(__spair_match_arm_local_view_state, #parent);
            }

        }};
        arm_code
    }

    fn generate_fn_create_closure_for_update(
        &self,
        sub_mod: &SubMod,
        _match_enum_name: &Ident,
        parent: &Ident,
    ) -> TokenStream {
        let view_state = Ident::new("__spair_local_view_state_for_update__", Span::call_site());
        let update_code = self.items.generate_fn_update(sub_mod, &view_state, parent);
        let fn_update_closure_name = &self.fn_update_ident;
        let arm_struct = sub_mod.generate(&self.arm_struct_view_name);
        let update_fn_closure = quote! {
            let #fn_update_closure_name = |#view_state: &mut #arm_struct, #parent| {
                #update_code
            };
        };
        update_fn_closure
    }

    fn generate_arm_view_states(&self) -> TokenStream {
        let view_state_name = &self.arm_struct_view_name;
        let fields = self
            .items
            .generate_view_state_struct_fields(&SubMod::new(None));

        let parent = Ident::new("__spair_parent_element_", Span::call_site());
        let fn_body_remove_from = self.items.generate_fn_remove_from(&parent);

        quote! {
            pub struct #view_state_name {
                #fields
            }
            impl #view_state_name {
                pub fn remove_from(&self, #parent: &::spair::WsElement) {
                    #fn_body_remove_from
                }
            }
        }
    }

    fn generate_remove_from_by_arm(&self, enum_name: &Ident) -> TokenStream {
        let variant_name = &self.match_enum_variant_for_arm;
        quote! {
            #enum_name::#variant_name(value) => value.remove_from(parent),
        }
    }
}

fn collect_variable_name_from_expr_call_args(
    args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
    errors: &mut MultiErrors,
) -> Vec<String> {
    let mut variables = Vec::new();
    for arg in args {
        match arg {
            Expr::Assign(expr_assign) => {
                match expr_as_ident(*expr_assign.left.clone(), "Expected an ident ") {
                    Ok(ident) => variables.push(ident.to_string()),
                    Err(e) => errors.combine(e),
                }
            }
            other_expr => errors.error_at(
                other_expr.span(),
                "Expected `name_to_use_in_fn_create = &some_variable_name`",
            ),
        }
    }
    variables
}
