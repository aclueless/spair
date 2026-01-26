use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Block, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Local, Pat, Result, ReturnType, Stmt,
    Type, Visibility, spanned::Spanned, token::Brace,
};

use crate::{
    MultiErrors,
    dom::{ItemCounter, Items, LastNode, SubMod, stage::StagePicker},
};

/// A view can have many different items which can be attached to the same HTML element
pub struct View {
    pub view_name: Ident,
    pub vis: Visibility,
    pub item_impl: ItemImpl,
    pub items: Items,

    pub span_to_report_empty: Span,
    pub sub_mod: SubMod,
}

impl View {
    pub fn from_item_impl(mut item_impl: ItemImpl) -> Result<Self> {
        let mut errors = MultiErrors::default();
        validate_view_impl(&item_impl, &mut errors);
        validate_view_fn(
            item_impl.items.first(),
            "create",
            &item_impl.brace_token,
            false,
            &mut errors,
        );
        validate_view_fn(
            item_impl.items.get(1),
            "update",
            &item_impl.brace_token,
            false,
            &mut errors,
        );
        validate_view_fn(
            item_impl.items.get(2),
            "view",
            &item_impl.brace_token,
            true,
            &mut errors,
        );
        let vis = check_vis(&item_impl, &mut errors);
        if let Some(item) = item_impl.items.get(3) {
            errors.error_at(
                item.span(),
                "Only expected 3 fn items, named: `create`, `update` and `view`",
            );
        }
        let ImplItem::Fn(view_fn) = item_impl.items.pop().unwrap() else {
            unreachable!("Checked by the last call to validate_view_fn above");
        };

        let view_name = match &*item_impl.self_ty {
            Type::Path(type_path) if type_path.path.get_ident().is_some() => {
                type_path.path.get_ident().unwrap().clone()
            }
            _ => {
                return errors.with_last_error_at(
                    item_impl.self_ty.span(),
                    "Type name must be a single identifier",
                );
            }
        };

        let update_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(item_impl.items.get(1))
                .iter()
                .map(|v| v.to_string())
                .collect();
        let stage_picker = StagePicker::CheckWithUpdateVariables(update_stage_variables);
        let span_to_report_empty = view_fn.block.span();
        let mut item_counter = ItemCounter::new(view_name.to_string());
        let sub_mod_name = item_counter.new_ident_sub_mod(&view_name);
        let mut items = Items::default();
        items.collect_from_block(
            view_fn.block,
            false,
            &stage_picker,
            &mut item_counter,
            &mut errors,
        );

        items.validate_html(&mut errors);

        let mut view = View {
            view_name,
            item_impl,
            vis,
            items,

            span_to_report_empty,
            sub_mod: SubMod::new(Some(sub_mod_name)),
        };
        view.check_create_variables_vs_update_variables(&mut errors);
        errors.report_error()?;
        view.items.prepare_items_for_generating_code();
        Ok(view)
    }

    fn check_create_variables_vs_update_variables(&self, errors: &mut MultiErrors) {
        let create_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(self.item_impl.items.first());
        let update_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(self.item_impl.items.get(1));
        check_for_name_conflicting(&create_stage_variables, &update_stage_variables, errors)
    }

    pub fn generate(&self) -> TokenStream {
        // the struct to store the state of this view
        let view_state_struct_for_self = self.generate_view_state_struct();

        // view for all `match` items in this view's DOM
        let view_states_for_matches_and_lists =
            self.items.generate_view_states_for_matches_and_lists();
        let view_states_for_matches_and_lists =
            self.sub_mod.generate_mod(view_states_for_matches_and_lists);

        let mut impl_view_state = self.item_impl.clone();

        // fn create(...) -> Self {...}
        self.generate_fn_create(
            impl_view_state
                .items
                .first_mut()
                .expect("get fn create for view"),
        );

        // fn update(&mut self, parent: &::spair::WsElement, ...) {...}
        self.generate_update_fn(
            impl_view_state
                .items
                .last_mut()
                .expect("get fn update view"),
        );

        // fn remove_from(&self, parent: &::spair::WsElement) {...}
        self.generate_fn_remove_from(&mut impl_view_state);

        quote! {
            #view_state_struct_for_self
            #view_states_for_matches_and_lists
            #impl_view_state
        }
    }

    fn generate_view_state_struct(&self) -> TokenStream {
        let vis = &self.vis;
        let view_state_struct_name = &self.view_name;
        let view_state_struct_fields = self.items.generate_view_state_struct_fields(&self.sub_mod);
        quote! {#vis struct #view_state_struct_name{#view_state_struct_fields}}
    }

    fn generate_fn_create(&self, fn_create: &mut ImplItem) {
        let ImplItem::Fn(fn_create) = fn_create else {
            return;
        };
        // add `pub` to make it `pub fn create`
        fn_create.vis = Visibility::Public(syn::token::Pub {
            span: Span::call_site(),
        });

        // insert parent input
        let parent_of_the_view = Ident::new("__spair_parent_of_the_view", Span::call_site());
        let parent_arg = quote! {#parent_of_the_view: &::spair::WsElement};
        let parent_arg: FnArg =
            syn::parse(parent_arg.into()).expect("macros::view::View::fn_create parent arg");
        fn_create.sig.inputs.insert(0, parent_arg);

        // insert next_sibling input
        let next_sibling_of_the_view =
            Ident::new("__spair_next_sibling_of_the_view", Span::call_site());
        let next_sibling_arg = quote! {#next_sibling_of_the_view: Option<&::spair::web_sys::Node>};
        let next_sibling_arg: FnArg = syn::parse(next_sibling_arg.into())
            .expect("macros::view::View::fn_create next_sibling arg");
        fn_create.sig.inputs.insert(1, next_sibling_arg);

        // add return type to make it `fn create(...) -> Self`
        let return_type = quote! {-> Self};
        let return_type: ReturnType =
            syn::parse(return_type.into()).expect("-> Self for fn create view");
        fn_create.sig.output = return_type;

        // insert `use ::spair::*` to the beginning of the fn create body
        insert_use_spair_items_to_fn(fn_create);

        // add create_fn_code to the end of the fn create body
        let fn_create_fn_body =
            self.generate_fn_create_fn_body(&parent_of_the_view, &next_sibling_of_the_view);
        // println!("{}", &fn_create_fn_body);
        let create_fn_code: Block =
            syn::parse(fn_create_fn_body.into()).expect("fn_create code for view");
        fn_create.block.stmts.extend(create_fn_code.stmts);
    }

    fn generate_fn_create_fn_body(
        &self,
        parent_of_the_view: &Ident,
        next_sibling_of_the_view: &Ident,
    ) -> TokenStream {
        let mut html_static_string = String::new();
        self.items.generate_html_string(&mut html_static_string);

        let template_fragment = Ident::new("_spair_view_document_fragment_", Span::call_site());
        let mut last_node = LastNode {
            parent: template_fragment.clone(),
            previous: None,
        };

        let create_elements_code = self.items.generate_fn_create(&self.sub_mod, &mut last_node);
        let view_state_instance_construction =
            self.generate_fn_create_return_value(&self.view_name);
        let create_fn_code = quote! {{
            const HTML_STRING: &str = #html_static_string;
            let #template_fragment = ::spair::TemplateElement::new(HTML_STRING).fragment();
            #create_elements_code
            #parent_of_the_view.insert_new_node_before_a_node(&#template_fragment, #next_sibling_of_the_view);
            #view_state_instance_construction
        }};
        create_fn_code
    }

    fn generate_fn_create_return_value(&self, view_name: &Ident) -> TokenStream {
        let fields = self.items.generate_fn_create_return_value();
        quote! {
            #view_name{
                #fields
            }
        }
    }

    fn generate_update_fn(&self, fn_update: &mut ImplItem) {
        let ImplItem::Fn(fn_update) = fn_update else {
            return;
        };

        let parent = Ident::new("_spair_web_sys_node_parent_", Span::call_site());

        // add `pub` to make it  `pub fn update`
        fn_update.vis = Visibility::Public(syn::token::Pub {
            span: fn_update.sig.fn_token.span,
        });

        // insert `parent: &::spair::web_sys::Node` to make it `fn update(parent: &::spair::web_sys::Node, ...)`
        if fn_update.sig.inputs.is_empty().not() {
            let arg_parent = quote! {#parent: &::spair::WsElement};
            let arg_parent: syn::FnArg = syn::parse(arg_parent.into())
                .expect("parent: &::spair::WsElement for fn update view");
            fn_update.sig.inputs.insert(0, arg_parent);
        }

        // insert `&mut self` to make it `fn update(&mut self, ...)`
        let mut_self = quote! {&mut self};
        let mut_self: syn::FnArg =
            syn::parse(mut_self.into()).expect("fn update(&mut self,...) for fn update view");
        fn_update.sig.inputs.insert(0, mut_self);

        // insert `use ::spair::*` to the beginning of the fn update body
        insert_use_spair_items_to_fn(fn_update);

        let view_state = Ident::new("_spair_self_this_me_view_state_", Span::call_site());
        let update_code = self
            .items
            .generate_fn_update(&self.sub_mod, &view_state, &parent);
        let update_code = quote! {{
            let #view_state = self;
            #update_code
        }};
        let update_code: Block = syn::parse(update_code.into()).expect("fn_update code for view");
        fn_update.block.stmts.extend(update_code.stmts);
    }

    fn generate_fn_remove_from(&self, item_impl: &mut ItemImpl) {
        let parent = Ident::new("_spair_parent_ws_element_", Span::call_site());
        let fn_body_remove_from = self.items.generate_fn_remove_from(&parent);
        let fn_remove_from = quote! {
            pub fn remove_from(&self, #parent: &::spair::WsElement) {
                #fn_body_remove_from
            }
        };
        let fn_remove_from: ImplItemFn =
            syn::parse(fn_remove_from.into()).expect("fn remove_from for view");

        item_impl.items.push(ImplItem::Fn(fn_remove_from));
    }
}

pub fn insert_use_spair_items_to_fn(fn_view: &mut ImplItemFn) {
    let import_spair_items = quote! {{
        use ::spair::{RenderOptionWithDefault, WsNodeFns};
    }};
    let mut import_spair_items: Block =
        syn::parse(import_spair_items.into()).expect("insert_use_spair_items_to_fn");
    while let Some(stmt) = import_spair_items.stmts.pop() {
        fn_view.block.stmts.insert(0, stmt);
    }
}

pub fn check_for_name_conflicting(
    create_stage_variables: &[Ident],
    update_stage_variables: &[Ident],
    errors: &mut MultiErrors,
) {
    for create_variable in create_stage_variables.iter() {
        for update_variable in update_stage_variables.iter() {
            if create_variable == update_variable {
                errors.error_at(
                    update_variable.span(),
                    "variable names in `fn create` must have names different from variable names in `fn update`",
                );
                errors.error_at(
                    create_variable.span(),
                    "variable names in `fn create` must have names different from variable names in `fn update`",
                );
            }
        }
    }
}

fn validate_view_impl(item_impl: &ItemImpl, errors: &mut MultiErrors) {
    if let Some(u) = item_impl.unsafety.as_ref() {
        errors.error_at(u.span(), "Not supported");
    }
    if item_impl.generics.lt_token.as_ref().is_some() {
        errors.error_at(item_impl.generics.span(), "Not supported");
    }
    if let Some(t) = item_impl.trait_.as_ref() {
        errors.error_at(t.1.span(), "Not supported");
    }
}

fn check_vis(item_impl: &ItemImpl, errors: &mut MultiErrors) -> Visibility {
    let Some(ImplItem::Fn(fn_update)) = item_impl.items.get(1) else {
        return Visibility::Inherited;
    };
    let Some(ImplItem::Fn(fn_view)) = item_impl.items.get(2) else {
        return Visibility::Inherited;
    };

    let vis_fn_create = get_vis(item_impl.items.first());
    let vis_fn_update = get_vis(item_impl.items.get(1));
    let vis_fn_view = get_vis(item_impl.items.get(2));
    if same_vis(&vis_fn_create, &vis_fn_update).not() {
        errors.error_at(
            if matches!(vis_fn_update, Visibility::Inherited) {
                fn_update.sig.fn_token.span()
            } else {
                vis_fn_update.span()
            },
            "Visibility of this fn must be the same as fn create",
        );
    }
    if same_vis(&vis_fn_create, &vis_fn_view).not() {
        errors.error_at(
            if matches!(vis_fn_view, Visibility::Inherited) {
                fn_view.sig.fn_token.span()
            } else {
                vis_fn_view.span()
            },
            "Visibility of this fn must be the same as fn create",
        );
    }
    vis_fn_view
}

fn same_vis(vis1: &Visibility, vis2: &Visibility) -> bool {
    match (vis1, vis2) {
        (Visibility::Public(_), Visibility::Public(_)) => true,
        (Visibility::Restricted(vis1), Visibility::Restricted(vis2)) => {
            if vis1.path.segments.len() != vis2.path.segments.len() {
                return false;
            }
            vis1.path
                .segments
                .iter()
                .zip(vis2.path.segments.iter())
                .all(|(p1, p2)| p1.ident == p2.ident)
        }
        (Visibility::Inherited, Visibility::Inherited) => true,
        _ => false,
    }
}

fn get_vis(impl_item: Option<&ImplItem>) -> Visibility {
    match impl_item {
        Some(ImplItem::Fn(iif)) => iif.vis.clone(),
        _ => Visibility::Inherited,
    }
}

fn validate_view_fn(
    impl_item: Option<&ImplItem>,
    expected_fn_name: &str,
    brace: &Brace,
    is_the_view_fn: bool,
    errors: &mut MultiErrors,
) {
    let message = format!("Expected `fn {expected_fn_name}(...) `");
    let Some(impl_item) = impl_item else {
        errors.error_at(brace.span.close(), &message);
        return;
    };
    let ImplItem::Fn(item_fn) = impl_item else {
        errors.error_at(impl_item.span(), &message);
        return;
    };
    if item_fn.sig.ident != expected_fn_name {
        errors.error_at(item_fn.sig.ident.span(), &message);
        return;
    }
    // if matches!(&item_fn.vis, Visibility::Inherited).not() {
    //     errors.error_at(item_fn.sig.span(), "Not supported");
    // }
    if item_fn.sig.abi.is_some() {
        errors.error_at(item_fn.sig.abi.span(), "Not supported");
    }
    if item_fn.sig.unsafety.is_some() {
        errors.error_at(item_fn.sig.unsafety.span(), "Not supported");
    }
    if item_fn.sig.asyncness.is_some() {
        errors.error_at(item_fn.sig.asyncness.span(), "Not supported");
    }
    if item_fn.sig.constness.is_some() {
        errors.error_at(item_fn.sig.constness.span(), "Not supported");
    }
    if let Some(receiver_arg) = item_fn.sig.receiver() {
        errors.error_at(receiver_arg.span(), "Not supported");
    }
    if item_fn.sig.generics.lt_token.as_ref().is_some() {
        errors.error_at(item_fn.sig.generics.span(), "Not supported");
    }
    if matches!(item_fn.sig.output, ReturnType::Default).not() {
        errors.error_at(item_fn.sig.output.span(), "Not supported");
    }
    if is_the_view_fn {
        if let Some(first) = item_fn.sig.inputs.first() {
            errors.error_at(first.span(), "Not supported");
        }
    } else {
        for input in item_fn.sig.inputs.iter() {
            match input {
                FnArg::Receiver(_) => {}
                FnArg::Typed(pat_type) => {
                    if matches!(&*pat_type.pat, Pat::Ident(_)).not() {
                        errors.error_at(
                            pat_type.pat.span(),
                            "Only support simple args: `arg_name: SomeTypeName`",
                        );
                    }
                }
            }
        }
    }
}

fn collect_variables_from_fn_sig_n_let_bindings(impl_item: Option<&ImplItem>) -> Vec<Ident> {
    let mut variable_names = Vec::new();
    if let Some(ImplItem::Fn(impl_item_fn)) = impl_item {
        for input in impl_item_fn.sig.inputs.iter() {
            if let FnArg::Typed(pat_type) = input {
                collect_variable_names_from_pat(&pat_type.pat, &mut variable_names);
            }
        }
        for stmt in impl_item_fn.block.stmts.iter() {
            if let Stmt::Local(Local { pat, .. }) = stmt {
                collect_variable_names_from_pat(pat, &mut variable_names);
            }
        }
    }

    variable_names
}

pub fn collect_variable_names_from_pat(pat: &Pat, variable_names: &mut Vec<Ident>) {
    match pat {
        Pat::Ident(pat_ident) => variable_names.push(pat_ident.ident.clone()),
        Pat::Reference(pat_reference) => {
            collect_variable_names_from_pat(&pat_reference.pat, variable_names);
        }
        Pat::Struct(pat_struct) => {
            for v in pat_struct.fields.iter() {
                collect_variable_names_from_pat(&v.pat, variable_names);
            }
        }
        Pat::Tuple(pat_tuple) => {
            for v in pat_tuple.elems.iter() {
                collect_variable_names_from_pat(v, variable_names);
            }
        }
        Pat::TupleStruct(pat_tuple_struct) => {
            for v in pat_tuple_struct.elems.iter() {
                collect_variable_names_from_pat(v, variable_names);
            }
        }
        Pat::Type(pat_type) => collect_variable_names_from_pat(&pat_type.pat, variable_names),
        _ => {}
    }
}
