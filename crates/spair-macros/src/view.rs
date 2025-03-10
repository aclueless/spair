use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::*;
use token::Brace;

use crate::element::{Element, HtmlElement};
use crate::{ItemCounter, MultiErrors};

pub(crate) struct View {
    pub(crate) view_state_type_name: Ident,
    pub(crate) item_impl: ItemImpl,
    pub(crate) element: HtmlElement,
}

impl View {
    pub(crate) fn with_item_impl(mut item_impl: ItemImpl) -> Result<Self> {
        let mut errors = MultiErrors::default();
        validate_view_impl(&item_impl, &mut errors);
        validate_view_fn(
            item_impl.items.get(0),
            "create_view",
            &item_impl.brace_token,
            false,
            &mut errors,
        );
        validate_view_fn(
            item_impl.items.get(1),
            "update_view",
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
        errors.report_error()?;

        let ImplItem::Fn(view) = item_impl.items.pop().unwrap() else {
            unreachable!("Checked by the last call to validate_view_fn above");
        };

        let view_state_name = match &*item_impl.self_ty {
            Type::Path(type_path) if type_path.path.get_ident().is_some() => {
                type_path.path.get_ident().unwrap().clone()
            }
            _ => {
                return Err(Error::new(
                    item_impl.self_ty.span(),
                    "Type name must be a single identifier",
                ));
            }
        };

        let update_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(item_impl.items.get(1))
                .iter()
                .map(|v| v.to_string())
                .collect();
        let element = collect_view_element(
            view.block,
            view_state_name.to_string(),
            &update_stage_variables,
        )?;
        element.validate_html()?;

        let mut view = View {
            view_state_type_name: view_state_name,
            item_impl,
            element,
        };
        view.check_create_variables_vs_update_variables()?;
        view.prepare_items_for_generating_code();
        Ok(view)
    }

    // which will be in update stage, which will be in create stage
    // should a variable be generated
    fn prepare_items_for_generating_code(&mut self) {
        let _ = self.element.prepare_items_for_generating_code();
    }

    fn check_create_variables_vs_update_variables(&self) -> Result<()> {
        let create_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(self.item_impl.items.get(0));
        let update_stage_variables: Vec<_> =
            collect_variables_from_fn_sig_n_let_bindings(self.item_impl.items.get(1));
        for create_variable in create_stage_variables.iter() {
            for update_variable in update_stage_variables.iter() {
                if create_variable == update_variable {
                    let mut error_in_update = syn::Error::new(update_variable.span(), "variable names in `fn create_with` must have names different from variable names in `fn update_with`");
                    let error_in_create= syn::Error::new(create_variable.span(), "variable names in `fn create_with` must have names different from variable names in `fn update_with`");
                    error_in_update.combine(error_in_create);
                    return Err(error_in_update);
                }
            }
        }
        Ok(())
    }
}

fn collect_variables_from_fn_sig_n_let_bindings(impl_item: Option<&ImplItem>) -> Vec<Ident> {
    fn collect_from_pat(pat: &Pat, update_stage_variables: &mut Vec<Ident>) {
        match pat {
            Pat::Ident(pat_ident) => update_stage_variables.push(pat_ident.ident.clone()),
            Pat::Reference(pat_reference) => {
                collect_from_pat(&pat_reference.pat, update_stage_variables);
            }
            Pat::Struct(pat_struct) => {
                for v in pat_struct.fields.iter() {
                    collect_from_pat(&v.pat, update_stage_variables);
                }
            }
            Pat::Tuple(pat_tuple) => {
                for v in pat_tuple.elems.iter() {
                    collect_from_pat(v, update_stage_variables);
                }
            }
            Pat::TupleStruct(pat_tuple_struct) => {
                for v in pat_tuple_struct.elems.iter() {
                    collect_from_pat(v, update_stage_variables);
                }
            }
            Pat::Type(pat_type) => collect_from_pat(&pat_type.pat, update_stage_variables),
            _ => {}
        }
    }
    let mut update_stage_variables = Vec::new();
    if let Some(ImplItem::Fn(impl_item_fn)) = impl_item {
        for input in impl_item_fn.sig.inputs.iter() {
            if let FnArg::Typed(pat_type) = input {
                collect_from_pat(&pat_type.pat, &mut update_stage_variables);
            }
        }
        for stmt in impl_item_fn.block.stmts.iter() {
            if let Stmt::Local(Local { pat, .. }) = stmt {
                collect_from_pat(pat, &mut update_stage_variables);
            }
        }
    }

    update_stage_variables
}

pub(crate) fn expr_has_ref_to(expr: &Expr, variable_names: &[String]) -> bool {
    match expr {
        Expr::Array(expr_array) => {
            for expr in expr_array.elems.iter() {
                if expr_has_ref_to(expr, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Assign(_expr_assign) => false,
        Expr::Async(_expr_async) => false,
        Expr::Await(_expr_await) => false,
        Expr::Binary(expr_binary) => {
            expr_has_ref_to(&expr_binary.left, variable_names)
                || expr_has_ref_to(&expr_binary.right, variable_names)
        }
        Expr::Block(expr_block) => block_has_ref_to(&expr_block.block, variable_names),
        Expr::Break(_expr_break) => false,
        Expr::Call(expr_call) => {
            for arg in expr_call.args.iter() {
                if expr_has_ref_to(arg, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Cast(expr_cast) => expr_has_ref_to(&expr_cast.expr, variable_names),
        Expr::Closure(expr_closure) => expr_has_ref_to(&expr_closure.body, variable_names),
        Expr::Const(_expr_const) => false,
        Expr::Continue(_expr_continue) => false,
        Expr::Field(expr_field) => expr_has_ref_to(&expr_field.base, variable_names),
        Expr::ForLoop(expr_for_loop) => expr_has_ref_to(&expr_for_loop.expr, variable_names),
        Expr::Group(_expr_group) => false,
        Expr::If(expr_if) => {
            expr_has_ref_to(&expr_if.cond, variable_names)
                || block_has_ref_to(&expr_if.then_branch, variable_names)
                || expr_if
                    .else_branch
                    .as_ref()
                    .map(|else_branch| expr_has_ref_to(&else_branch.1, variable_names))
                    .unwrap_or_default()
        }
        Expr::Index(expr_index) => {
            expr_has_ref_to(&expr_index.expr, variable_names)
                || expr_has_ref_to(&expr_index.index, variable_names)
        }
        Expr::Infer(_expr_infer) => false,
        Expr::Let(expr_let) => expr_has_ref_to(&expr_let.expr, variable_names),
        Expr::Lit(_expr_lit) => false,
        Expr::Loop(expr_loop) => block_has_ref_to(&expr_loop.body, variable_names),
        Expr::Macro(_expr_macro) => true,
        Expr::Match(expr_match) => expr_has_ref_to(&expr_match.expr, variable_names),
        Expr::MethodCall(expr_method_call) => {
            if expr_has_ref_to(&expr_method_call.receiver, variable_names) {
                return true;
            }
            for a in expr_method_call.args.iter() {
                if expr_has_ref_to(a, variable_names) {
                    return false;
                }
            }
            false
        }
        Expr::Paren(expr_paren) => expr_has_ref_to(&expr_paren.expr, variable_names),
        Expr::Path(expr_path) => {
            if expr_path.qself.is_some() {
                return false;
            }
            if expr_path.path.segments.len() != 1 {
                return false;
            }
            for vname in variable_names.iter() {
                if expr_path.path.is_ident(vname) {
                    return true;
                }
            }
            false
        }
        Expr::Range(expr_range) => {
            if let Some(expr) = expr_range.start.as_ref() {
                if expr_has_ref_to(&expr, variable_names) {
                    return true;
                }
            }
            if let Some(expr) = expr_range.end.as_ref() {
                if expr_has_ref_to(&expr, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::RawAddr(_expr_raw_addr) => false,
        Expr::Reference(expr_reference) => expr_has_ref_to(&expr_reference.expr, variable_names),
        Expr::Repeat(expr_repeat) => expr_has_ref_to(&expr_repeat.expr, variable_names),
        Expr::Return(expr_return) => expr_return
            .expr
            .as_ref()
            .map(|v| expr_has_ref_to(v, variable_names))
            .unwrap_or(false),
        Expr::Struct(expr_struct) => {
            for f in expr_struct.fields.iter() {
                if expr_has_ref_to(&f.expr, variable_names) {
                    return true;
                }
            }
            if let Some(expr) = expr_struct.rest.as_ref() {
                expr_has_ref_to(expr, variable_names)
            } else {
                false
            }
        }
        Expr::Try(expr_try) => expr_has_ref_to(&expr_try.expr, variable_names),
        Expr::TryBlock(expr_try_block) => block_has_ref_to(&expr_try_block.block, variable_names),
        Expr::Tuple(expr_tuple) => {
            for expr in expr_tuple.elems.iter() {
                if expr_has_ref_to(expr, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Unary(expr_unary) => expr_has_ref_to(&expr_unary.expr, variable_names),
        Expr::Unsafe(expr_unsafe) => block_has_ref_to(&expr_unsafe.block, variable_names),
        Expr::Verbatim(_token_stream) => false,
        Expr::While(expr_while) => {
            expr_has_ref_to(&expr_while.cond, variable_names)
                || block_has_ref_to(&expr_while.body, variable_names)
        }
        Expr::Yield(_expr_yield) => false,
        _ => false,
    }
}

fn block_has_ref_to(block: &Block, update_stage_variables: &[String]) -> bool {
    for stmt in block.stmts.iter() {
        if let Stmt::Expr(expr, _) = stmt {
            if expr_has_ref_to(expr, update_stage_variables) {
                return true;
            }
        }
    }
    false
}

fn collect_view_element(
    mut block: Block,
    match_view_state_prefix: String,
    update_stage_variables: &[String],
) -> Result<HtmlElement> {
    if block.stmts.is_empty() {
        return Err(Error::new(block.span(), "A view can not be empty"));
    }
    let message_one_element_only = "This is the second statement. Spair requires the HTML construction statement is the only and last statement a view or component";
    if let Some(second_stmt) = block.stmts.get(1) {
        return Err(syn::Error::new(
            second_stmt.span(),
            message_one_element_only,
        ));
    }
    let message_html_element_only = "Spair view only supports an HTML element as root node";
    let mut item_counter = ItemCounter::new(match_view_state_prefix);
    match block.stmts.remove(0) {
        Stmt::Expr(expr, _) => {
            let span = expr.span();
            let mut elements =
                Element::with_expr(expr, &mut item_counter, Some(update_stage_variables))?;
            if elements.is_empty() {
                return Err(Error::new(span, "A view can not be empty"));
            }
            let element = elements.remove(0);
            let html_element = match element {
                Element::Text(text) => Err(syn::Error::new(
                    text.shared_name.span(),
                    message_html_element_only,
                )),
                Element::HtmlElement(mut html_element) => {
                    html_element.root_element = true;
                    Ok(html_element)
                }
                Element::View(view) => {
                    Err(syn::Error::new(view.name.span(), message_html_element_only))
                }
                Element::KeyedList(list) => {
                    Err(syn::Error::new(list.name.span(), message_html_element_only))
                }
                Element::Match(m) => Err(syn::Error::new(
                    m.match_keyword.span(),
                    message_html_element_only,
                )),
            }?;
            if let Some(second) = elements.first() {
                return Err(syn::Error::new(
                    second.name_or_text_expr_span(),
                    message_one_element_only,
                ));
            }
            return Ok(html_element);
        }
        stmt => Err(syn::Error::new(stmt.span(), message_html_element_only)),
    }
}

fn validate_view_impl(item_impl: &ItemImpl, errors: &mut MultiErrors) {
    if let Some(u) = item_impl.unsafety.as_ref() {
        errors.add(u.span(), "Spair view does not support unsafe");
    }
    if let Some(_) = item_impl.generics.lt_token.as_ref() {
        errors.add(
            item_impl.generics.span(),
            "Spair view does not support generics",
        );
    }
    if let Some(t) = item_impl.trait_.as_ref() {
        errors.add(t.1.span(), "Spair view does not support trait impl");
    }
}

fn validate_view_fn(
    impl_item: Option<&ImplItem>,
    expected_name: &str,
    brace: &Brace,
    is_the_view_fn: bool,
    errors: &mut MultiErrors,
) {
    let message = format!("Expected `fn {expected_name}(...) `");
    let Some(impl_item) = impl_item else {
        errors.add(brace.span.close(), &message);
        return;
    };
    let ImplItem::Fn(item_fn) = impl_item else {
        errors.add(impl_item.span(), &message);
        return;
    };
    if item_fn.sig.ident != expected_name {
        errors.add(item_fn.sig.ident.span(), &message);
        return;
    }
    if matches!(&item_fn.vis, Visibility::Inherited).not() {
        errors.add(
            item_fn.sig.span(),
            "Do not give `pub` here, the macro will add it if it is required",
        );
    }
    if item_fn.sig.abi.is_some() {
        errors.add(
            item_fn.sig.abi.span(),
            "Spair does not support views on extern C functions",
        );
    }
    if item_fn.sig.unsafety.is_some() {
        errors.add(
            item_fn.sig.unsafety.span(),
            "Spair does not support views on unsafe functions",
        );
    }
    if item_fn.sig.asyncness.is_some() {
        errors.add(
            item_fn.sig.asyncness.span(),
            "Spair does not support views on async functions",
        );
    }
    if item_fn.sig.constness.is_some() {
        errors.add(
            item_fn.sig.constness.span(),
            "Spair does not support views on const functions",
        );
    }
    if let Some(receiver_arg) = item_fn.sig.receiver() {
        errors.add(
            receiver_arg.span(),
            "Spair does not support views on functions with receiver parameters",
        );
    }
    if let Some(_) = item_fn.sig.generics.lt_token.as_ref() {
        errors.add(
            item_fn.sig.generics.span(),
            "Spair does not support views on functions with generics",
        );
    }
    if matches!(item_fn.sig.output, ReturnType::Default).not() {
        errors.add(
            item_fn.sig.output.span(),
            "Do not give a return type, the macro will add a return type if the method need it",
        );
    }
    if is_the_view_fn {
        if let Some(first) = item_fn.sig.inputs.first() {
            errors.add(
                first.span(),
                "Spair expects args on `fn create_with` or `fn update_with`, don't put args on `fn view`.",
            );
        }
    } else {
        for input in item_fn.sig.inputs.iter() {
            match input {
                FnArg::Receiver(_) => {}
                FnArg::Typed(pat_type) => {
                    if matches!(&*pat_type.pat, Pat::Ident(_)).not() {
                        errors.add(
                            pat_type.pat.span(),
                            "Spair only supports simple args: `arg_name: SomeTypeName`",
                        );
                    }
                }
            }
        }
    }
}

impl View {
    fn generate_impl_view_state_fns(&self) -> TokenStream {
        let generated_create_view_fn =
            self.generate_impl_create_view_fn(&self.view_state_type_name);
        let generated_update_view_fn = self.generate_impl_update_view_fn();

        let impl_token = &self.item_impl.impl_token;
        let self_type = &self.item_impl.self_ty;
        let root_element_type = self.element.root_element_type();
        let root_element_ident = self.element.root_element_ident();
        quote! {
            #impl_token #self_type {
                #generated_create_view_fn
                #generated_update_view_fn
                pub fn root_element(&self) -> &#root_element_type {
                    &self.#root_element_ident
                }
            }
        }
    }

    fn generate_impl_create_view_fn(&self, view_state_struct_name: &Ident) -> TokenStream {
        let impl_item = self.item_impl.items.get(0);
        let fn_body = self
            .element
            .generate_code_for_create_view_fn_of_a_view(view_state_struct_name);
        let ImplItem::Fn(ImplItemFn { sig, block, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        let block = &block.stmts;
        quote! {
            pub #sig -> Self {
                #(#block)*
                #fn_body
            }
        }
    }

    fn generate_impl_update_view_fn(&self) -> TokenStream {
        let view_state_ident =
            Ident::new("_spair_view_view_state_for_updating_", Span::call_site());
        let impl_item = self.item_impl.items.get(1);
        let fn_body = self
            .element
            .generate_code_for_update_view_fn(&view_state_ident);
        let ImplItem::Fn(ImplItemFn { sig, block, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        let Signature {
            fn_token,
            ident,
            inputs,
            ..
        } = sig;
        let block = &block.stmts;
        quote! {
            pub #fn_token #ident(&mut self, #inputs) {
                #(#block)*
                let #view_state_ident = self;
                #fn_body
            }
        }
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = &self.view_state_type_name;
        let struct_fields = self.element.generate_view_state_struct_fields();
        let view_state_struct = quote! {pub struct #struct_name{#struct_fields}};
        let match_view_state_types = self.element.collect_match_view_state_types();
        let impl_view_state = self.generate_impl_view_state_fns();
        tokens.append_all([view_state_struct, match_view_state_types, impl_view_state]);
    }
}
