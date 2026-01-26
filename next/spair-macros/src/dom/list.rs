use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Block, Expr, ExprClosure, Ident, Local, Pat, PatType, ReturnType, Stmt, Type,
    spanned::Spanned,
    token::{self},
};

use crate::{MultiErrors, view::collect_variable_names_from_pat};

use super::{
    Element, ItemCounter, LastNode, SubMod, expr_as_ident,
    stage::{Stage, StagePicker},
};

pub struct List {
    spair_list_keyword: Ident,
    stage: Stage,
    items_iterator: Expr,
    keyed_list_items: Option<KeyedListItems>,
    create_view_closure: ExprClosure,
    update_view_closure: ExprClosure,
    element: Element,
    partial_list: bool,

    view_state_struct_name: Ident,
    spair_ident: Ident,
    spair_ident_marker: Ident,
}

struct KeyedListItems {
    key_type_name: Box<Type>,
    get_key_closure: ExprClosure,
}

impl std::fmt::Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "list")
    }
}

impl List {
    pub fn first_span(&self) -> proc_macro2::Span {
        self.spair_list_keyword.span()
    }

    pub fn new(
        at_root: bool,
        spair_list_keyword: Ident,
        paren_token: token::Paren,
        args: syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) -> Option<Self> {
        if at_root {
            errors.error_at(spair_list_keyword.span(), "Not allow at root level");
        }
        if args.len() != 4 && args.len() != 5 {
            errors.error_at(
                paren_token.span.join(),
                "Expected 4 items for a non-keyed list, or 5 items for a keyed list, like `spair_list(list_iterator, (optional) get key closure, create closure, update closure, element for view)`",
            );
            return None;
        }
        let is_keyed_list = args.len() == 5;

        let mut args = args.into_pairs();
        let Some(items_iterator) = args.next() else {
            errors.error_at(
                paren_token.span.close(),
                "Expected a items-iterator like: spair_list(some_list.iter(), ...)",
            );
            return None;
        };
        let keyed_list_items = if is_keyed_list {
            let Some(get_key_closure) = args.next() else {
                errors.error_at(
                paren_token.span.close(),
                "Expected a get-key-from-item-closure like: spair_list(..., |item| -> &KeyType {&item.key}, ...)",
                );
                return None;
            };
            KeyedListItems::from_closure(get_key_closure.into_value(), errors)
        } else {
            None
        };
        let Some(expr) = args.next() else {
            errors.error_at(
                paren_token.span.close(),
                "Expected a create-closure like: spair_list(..., |item_data_to_render_at_creation| -> {}, ...)",
            );
            return None;
        };
        let create_view_closure = match expr.into_value() {
            Expr::Closure(create_view_closure) => create_view_closure,
            other => {
                errors.error_at(
                other.span(),
                "Expected a create-closure like: spair_list(..., |item_data_to_render_at_creation| -> {}, ...)",
            );
                return None;
            }
        };
        let Some(expr) = args.next() else {
            errors.error_at(
                paren_token.span.close(),
                "Expected a create-closure like: spair_list(..., |item_data_to_render_in_update| -> {}, ...)",
            );
            return None;
        };
        let update_view_closure = match expr.into_value() {
            Expr::Closure(update_view_closure) => update_view_closure,
            other => {
                errors.error_at(
                other.span(),
                "Expected a create-closure like: spair_list(..., |item_data_to_render_in_update| -> {}, ...)",
            );
                return None;
            }
        };
        let Some(expr) = args.next() else {
            errors.error_at(
                paren_token.span.close(),
                "Expected an HTML element for item view like:  spair_list(..., div(...))",
            );
            return None;
        };
        let expr = match expr.into_value() {
            Expr::Call(expr) => expr,
            other => {
                errors.error_at(
                    other.span(),
                    "Expected an HTML element for item view like:  spair_list(..., div(...))",
                );
                return None;
            }
        };
        let element_name = match expr_as_ident(*expr.func, "Expected HTML tag") {
            Ok(ident) => ident,
            Err(e) => {
                errors.combine(e);
                return None;
            }
        };

        let items_iterator = items_iterator.into_value();
        let stage = stage_picker.stage_of(&items_iterator);

        let create_variables =
            collect_variables_from_closure_inputs_n_let_bindings(&create_view_closure);
        let stage_picker = StagePicker::CheckWithCreationVariables(
            create_variables.iter().map(|v| v.to_string()).collect(),
        );

        let element = Element::new(
            true,
            element_name,
            expr.args,
            &stage_picker,
            item_counter,
            errors,
        );
        Some(List {
            spair_list_keyword,
            stage,
            items_iterator,
            keyed_list_items,
            create_view_closure,
            update_view_closure,
            element,
            partial_list: false,
            view_state_struct_name: item_counter.new_list_struct(),
            spair_ident: item_counter.new_ident_list(),
            spair_ident_marker: item_counter.new_ident_marker("list"),
        })
    }

    pub fn validate_html(&self, errors: &mut crate::MultiErrors) {
        self.element.validate_html(errors);
    }

    pub fn prepare_items_for_generating_code(&mut self, parent_has_only_one_child: bool) {
        self.partial_list = parent_has_only_one_child.not();
        self.element.prepare_items_for_generating_code();
    }

    pub fn generate_view_state_struct_fields(&self, sub_mod: &SubMod) -> TokenStream {
        let ident = &self.spair_ident;
        let view_state_struct_name = sub_mod.generate(&self.view_state_struct_name);
        if let Some(key_items) = self.keyed_list_items.as_ref() {
            let key_type_name = &key_items.key_type_name;
            quote! {#ident: ::spair::KeyedList<#key_type_name,#view_state_struct_name>,}
        } else {
            quote! {#ident: ::spair::List<#view_state_struct_name>,}
        }
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        if self.partial_list {
            html_string.push_str("<!--iplist-->");
        }
    }

    pub fn generate_view_states_for_matches_and_lists(&self) -> TokenStream {
        let view_state = self.generate_item_view_state();
        let inner_view_states = self.element.generate_view_states_for_matches_and_lists();
        quote! {
            #view_state
            #inner_view_states
        }
    }

    fn generate_item_view_state(&self) -> TokenStream {
        let fields = self
            .element
            .generate_view_state_struct_fields(&SubMod::new(None));
        let view_state = &self.view_state_struct_name;
        let root_element = self.element.spair_ident();
        quote! {
            pub struct #view_state{
                #fields
            }

            impl ::spair::ItemViewState for #view_state {
                fn root_element(&self) -> &::spair::WsElement{
                    &self.#root_element
                }
            }
        }
    }

    pub fn generate_fn_create(&self, sub_mod: &SubMod, last_node: &LastNode) -> TokenStream {
        let parent = &last_node.parent;
        let ident = &self.spair_ident;

        let marker_ident = &self.spair_ident_marker;
        let get_end_node = if self.partial_list {
            let get_marker = last_node.get_ws_node(&self.spair_ident_marker);
            quote! {
                #get_marker
                let #marker_ident = Some(#marker_ident.get_ws_node_ref().clone());
            }
        } else {
            quote! {
                let #marker_ident = None;
            }
        };

        let mut html_string = String::new();
        self.element.generate_html_string(&mut html_string);

        let creat_list = if let Some(_key_items) = self.keyed_list_items.as_ref() {
            quote! {let #ident = ::spair::KeyedList::new(&#parent, #marker_ident, #html_string);}
        } else {
            quote! {let #ident = ::spair::List::new(&#parent, #marker_ident, #html_string);}
        };

        let render_at_creation = if self.stage == Stage::Creation {
            let fn_create = self.generate_create_closure(sub_mod);
            let fn_update = self.generate_update_closure(sub_mod);
            let items_iterator = &self.items_iterator;
            // quote! {
            //     let mut #ident = #ident;
            //     #ident.update(#items_iterator, #fn_create, #fn_update);
            // }
            if let Some(key_items) = self.keyed_list_items.as_ref() {
                let get_key_closure = &key_items.get_key_closure;
                quote! {
                    let mut #ident = #ident;
                    #ident.update(#items_iterator, #get_key_closure, #fn_create, #fn_update);
                }
            } else {
                quote! {
                    let mut #ident = #ident;
                    #ident.update(#items_iterator, #fn_create, #fn_update);
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #get_end_node
            #creat_list
            #render_at_creation
            let #marker_ident = #ident.end_node();
        }
    }

    fn generate_create_closure(&self, sub_mod: &SubMod) -> TokenStream {
        let mut fn_create = self.create_view_closure.clone();

        let template = Ident::new("__spair_template_element_", Span::call_site());

        let fn_arg = quote! {#template: ::spair::DocumentFragment};
        let pat_type: PatType =
            syn::parse(fn_arg.into()).expect("#template: ::spair::DocumentFragment");
        fn_create.inputs.insert(0, Pat::Type(pat_type));

        let last_node = LastNode {
            parent: template,
            previous: None,
        };
        let fn_create_code = self.element.generate_fn_create(sub_mod, &last_node);

        let return_value_fields = self.element.generate_fn_create_return_value();
        let view_name = sub_mod.generate(&self.view_state_struct_name);
        let return_value = quote! {
            #view_name {
                #return_value_fields
            }
        };

        let fn_create_code = quote! {{
            #fn_create_code
            #return_value
        }};

        let fn_create_code: Block =
            syn::parse(fn_create_code.into()).expect("fn_create_code for create closure");
        if let Expr::Block(block) = fn_create.body.as_mut() {
            block.block.stmts.extend(fn_create_code.stmts);
        }

        quote! {
            #fn_create
        }
    }

    fn generate_update_closure(&self, sub_mod: &SubMod) -> TokenStream {
        let mut fn_update = self.update_view_closure.clone();

        let view_name = sub_mod.generate(&self.view_state_struct_name);
        let view_state = Ident::new("__spair_view_state_", Span::call_site());

        let fn_arg = quote! {#view_state: &mut #view_name};
        let pat_type: PatType =
            syn::parse(fn_arg.into()).expect("#view_state: &mut #sub_mod_name::#view_name");
        fn_update.inputs.insert(0, Pat::Type(pat_type));

        let fn_update_code = self.element.generate_fn_update(sub_mod, &view_state);
        let fn_update_code = quote! {{ #fn_update_code}};

        let fn_update_code: Block =
            syn::parse(fn_update_code.into()).expect("fn_update_code for update closure");

        if let Expr::Block(block) = fn_update.body.as_mut() {
            block.block.stmts.extend(fn_update_code.stmts);
        }

        quote! {
            #fn_update
        }
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident_marker
    }

    pub fn generate_return_value(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {#ident,}
    }

    pub fn generate_fn_update(
        &self,
        sub_mod: &SubMod,
        view_state: &Ident,
        _parent: &Ident,
    ) -> TokenStream {
        if self.stage != Stage::Update {
            return quote! {};
        }

        let ident = &self.spair_ident;
        let items_iterator = &self.items_iterator;
        let fn_create = self.generate_create_closure(sub_mod);
        let fn_update = self.generate_update_closure(sub_mod);

        if let Some(key_items) = self.keyed_list_items.as_ref() {
            let get_key_closure = &key_items.get_key_closure;
            quote! {
                #view_state.#ident.update(#items_iterator, #get_key_closure, #fn_create, #fn_update);
            }
        } else {
            quote! {
                #view_state.#ident.update(#items_iterator, #fn_create, #fn_update);
            }
        }
    }
}

impl KeyedListItems {
    fn from_closure(expr: Expr, errors: &mut MultiErrors) -> Option<KeyedListItems> {
        let Expr::Closure(closure) = expr else {
            errors.error_at(
                expr.span(),
                "Expected a closure that returns the key of an item in the list",
            );
            return None;
        };
        let return_type = match &closure.output {
            ReturnType::Default => {
                errors.error_at(
                    closure.span(),
                    "Please specify a return type for this closure. The return type of this closure is the type of item's key.",
                );
                return None;
            }
            ReturnType::Type(_, type_name) => type_name.clone(),
        };

        let return_type = match *return_type {
            Type::Reference(type_reference) => type_reference.elem,
            other => {
                errors.error_at(
                    other.span(),
                    "Return type of `get_key` closure must be a reference like `&TypeName`",
                );
                return None;
            }
        };

        Some(KeyedListItems {
            key_type_name: return_type,
            get_key_closure: closure,
        })
    }
}

fn collect_variables_from_closure_inputs_n_let_bindings(closure: &ExprClosure) -> Vec<Ident> {
    let mut variable_names = Vec::new();
    for input in closure.inputs.iter() {
        collect_variable_names_from_pat(input, &mut variable_names);
    }
    if let Expr::Block(block) = closure.body.as_ref() {
        for stmt in block.block.stmts.iter() {
            if let Stmt::Local(Local { pat, .. }) = stmt {
                collect_variable_names_from_pat(pat, &mut variable_names);
            }
        }
    }

    variable_names
}
