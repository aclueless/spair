use std::ops::Not;

use comp_ref::CompRef;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use stage::StagePicker;
use syn::{Expr, ExprCall, Ident, Result, Stmt, spanned::Spanned};

pub use element::Element;
use list::List;
use match_expr::Match;
use text::Text;
pub use view::View;
use view::ViewFnCall;

use crate::MultiErrors;

mod comp_ref;
mod element;
mod list;
mod match_expr;
pub mod stage;
mod text;
mod view;

/// A list of items that can be attached to the same HTML element
#[derive(Default)]
pub struct Items {
    items: Vec<Item>,
}

impl std::fmt::Debug for Items {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "children {}", self.items.len())
    }
}

/// A text node, a list, a view, an HMTL element...
#[derive(Debug)]
pub enum Item {
    Text(Text),
    Element(Element),
    View(View),
    List(List),
    Match(Match),
    CompRef(CompRef),
}

pub struct SubMod {
    sub_mod_name: Option<Ident>,
}

impl SubMod {
    pub fn new(sub_mod_name: Option<Ident>) -> Self {
        Self { sub_mod_name }
    }

    pub fn generate(&self, ident: &Ident) -> TokenStream {
        match self.sub_mod_name.as_ref() {
            Some(sub_mod_name) => quote! {#sub_mod_name::#ident},
            None => quote! {#ident},
        }
    }

    pub fn generate_mod(&self, view_states_for_matches_and_lists: TokenStream) -> TokenStream {
        match self.sub_mod_name.as_ref() {
            Some(sub_mod_name) if view_states_for_matches_and_lists.is_empty().not() => {
                quote! {
                    mod #sub_mod_name {
                        #view_states_for_matches_and_lists
                    }
                }
            }
            _ => view_states_for_matches_and_lists,
        }
    }
}

pub struct LastNode {
    pub parent: Ident,
    pub previous: Option<Ident>,
}

impl LastNode {
    fn get_item(
        &self,
        ident: &Ident,
        first_item_method_name: &str,
        next_item_method_name: &str,
    ) -> TokenStream {
        let first_item_method_name = Ident::new(first_item_method_name, Span::call_site());
        let next_item_method_name = Ident::new(next_item_method_name, Span::call_site());
        match self.previous.as_ref() {
            None => {
                let parent = &self.parent;
                quote! { let #ident = #parent.#first_item_method_name(); }
            }
            Some(previous) => {
                quote! { let #ident = #previous.#next_item_method_name(); }
            }
        }
    }

    fn get_ws_node(&self, ident: &Ident) -> TokenStream {
        self.get_item(ident, "first_ws_node", "next_sibling_ws_node")
    }

    fn get_ws_text(&self, ident: &Ident) -> TokenStream {
        self.get_item(ident, "first_ws_text", "next_sibling_ws_text")
    }

    fn get_text(&self, ident: &Ident) -> TokenStream {
        self.get_item(ident, "first_text", "next_sibling_text")
    }

    fn get_ws_element(&self, ident: &Ident) -> TokenStream {
        self.get_item(ident, "first_ws_element", "next_sibling_ws_element")
    }
}
impl Item {
    pub fn first_span(&self) -> Span {
        match self {
            Item::Text(value) => value.first_span(),
            Item::Element(value) => value.first_span(),
            Item::View(value) => value.first_span(),
            Item::List(value) => value.first_span(),
            Item::Match(value) => value.first_span(),
            Item::CompRef(value) => value.first_span(),
        }
    }

    fn check_html_multi_errors(&self, errors: &mut MultiErrors) {
        match self {
            Item::Text(_value) => {}
            Item::Element(value) => value.validate_html(errors),
            Item::View(_value) => {}
            Item::List(value) => value.validate_html(errors),
            Item::Match(value) => value.validate_html(errors),
            Item::CompRef(_value) => {}
        }
    }

    fn prepare_items_for_generating_code(&mut self, parent_has_only_one_child: bool) {
        match self {
            Item::Text(_value) => {}
            Item::Element(value) => value.prepare_items_for_generating_code(),
            Item::View(_value) => {}
            Item::List(value) => value.prepare_items_for_generating_code(parent_has_only_one_child),
            Item::Match(value) => {
                value.prepare_items_for_generating_code(parent_has_only_one_child)
            }
            Item::CompRef(_value) => {}
        }
    }

    fn generate_view_state_struct_fields(&self, sub_mod: &SubMod) -> TokenStream {
        match self {
            Item::Text(value) => value.generate_view_state_struct_fields(),
            Item::Element(value) => value.generate_view_state_struct_fields(sub_mod),
            Item::View(value) => {
                value.generate_view_state_struct_fields(sub_mod.sub_mod_name.is_none())
            }
            Item::List(value) => value.generate_view_state_struct_fields(sub_mod),
            Item::Match(value) => value.generate_view_state_struct_fields(sub_mod),
            Item::CompRef(value) => value.generate_view_state_struct_fields(),
        }
    }

    fn generate_view_states_for_matches_and_lists(&self) -> TokenStream {
        match self {
            Item::Text(_value) => quote! {},
            Item::Element(value) => value.generate_view_states_for_matches_and_lists(),
            Item::View(_value) => quote! {},
            Item::List(value) => value.generate_view_states_for_matches_and_lists(),
            Item::Match(value) => value.generate_view_states_for_matches_and_lists(),
            Item::CompRef(_value) => quote! {},
        }
    }

    fn generate_html_string(&self, html_string: &mut String) {
        match self {
            Item::Text(value) => value.generate_html_string(html_string),
            Item::Element(value) => value.generate_html_string(html_string),
            Item::View(value) => value.generate_html_string(html_string),
            Item::List(value) => value.generate_html_string(html_string),
            Item::Match(value) => value.generate_html_string(html_string),
            Item::CompRef(value) => value.generate_html_string(html_string),
        }
    }

    fn generate_fn_create(&self, sub_mod: &SubMod, last_node: &LastNode) -> TokenStream {
        match self {
            Item::Text(value) => value.generate_fn_create(last_node),
            Item::Element(value) => value.generate_fn_create(sub_mod, last_node),
            Item::View(value) => value.generate_fn_create(last_node),
            Item::List(value) => value.generate_fn_create(sub_mod, last_node),
            Item::Match(value) => value.generate_fn_create(sub_mod, last_node),
            Item::CompRef(value) => value.generate_fn_create(last_node),
        }
    }

    fn spair_ident_to_get_next_node(&self) -> &Ident {
        match self {
            Item::Text(value) => value.spair_indent_to_get_next_node(),
            Item::Element(value) => value.spair_indent_to_get_next_node(),
            Item::View(value) => value.spair_indent_to_get_next_node(),
            Item::List(value) => value.spair_indent_to_get_next_node(),
            Item::Match(value) => value.spair_indent_to_get_next_node(),
            Item::CompRef(value) => value.spair_indent_to_get_next_node(),
        }
    }

    fn generate_fn_create_return_value(&self) -> TokenStream {
        match self {
            Item::Text(value) => value.generate_fn_create_return_value(),
            Item::Element(value) => value.generate_fn_create_return_value(),
            Item::View(value) => value.generate_fn_create_return_value(),
            Item::List(value) => value.generate_return_value(),
            Item::Match(value) => value.generate_fn_create_return_value(),
            Item::CompRef(value) => value.generate_fn_create_return_value(),
        }
    }

    fn generate_fn_update(
        &self,
        sub_mod: &SubMod,
        view_state: &Ident,
        parent: &Ident,
    ) -> TokenStream {
        match self {
            Item::Text(value) => value.generate_fn_update(view_state),
            Item::Element(value) => value.generate_fn_update(sub_mod, view_state),
            Item::View(value) => value.generate_fn_update(view_state, parent),
            Item::List(value) => value.generate_fn_update(sub_mod, view_state, parent),
            Item::Match(value) => value.generate_fn_update(sub_mod, view_state, parent),
            Item::CompRef(value) => value.generate_fn_update(view_state, parent),
        }
    }

    fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        match self {
            Item::Text(value) => value.generate_fn_remove_from(parent),
            Item::Element(value) => value.generate_fn_remove_from(parent),
            Item::View(value) => value.generate_fn_remove_from(parent),
            Item::List(_value) => {
                panic!("List must not be at the root of a view");
            }
            Item::Match(value) => value.generate_fn_remove_from(parent),
            Item::CompRef(value) => value.generate_fn_remove_from(parent),
        }
    }
}

impl Items {
    pub fn collect_from_block(
        &mut self,
        block: syn::Block,
        allow_empty: bool,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) {
        let span = block.span();
        for stmt in block.stmts {
            match stmt {
                Stmt::Expr(expr, _) => {
                    self.collect_from_expr(true, expr, stage_picker, item_counter, errors);
                }
                stmt => {
                    errors.error_at(stmt.span(), "Expect HTML elements, views, texts");
                }
            }
        }
        if allow_empty.not() && self.items.is_empty() {
            errors.error_at(span, "Expect at least one of HTML element, view or text");
        }
    }

    fn collect_from_expr(
        &mut self,
        at_root: bool,
        expr: Expr,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) {
        match expr {
            Expr::Call(expr_call) => {
                self.collect_from_expr_call(at_root, expr_call, stage_picker, item_counter, errors);
            }
            Expr::Field(expr_field) => {
                self.collect_text_node(None, stage_picker, item_counter, Expr::Field(expr_field));
            }
            Expr::Lit(expr_lit) => {
                self.collect_literal_text_node(None, item_counter, errors, expr_lit);
            }
            Expr::Match(expr_match) => {
                self.items.push(Item::Match(Match::with_expr_match(
                    expr_match,
                    stage_picker,
                    item_counter,
                    errors,
                )));
            }
            Expr::MethodCall(mcall) => {
                if mcall.method == "update" {
                    if let Expr::Call(expr_call) = mcall.receiver.as_ref() {
                        if let Expr::Path(path) = &expr_call.func.as_ref() {
                            if let Some(view_name) = path.path.get_ident() {
                                if is_first_letter_uppercase(&view_name.to_string()) {
                                    let update_call = ViewFnCall::new(
                                        mcall.method,
                                        mcall.paren_token,
                                        mcall.args,
                                    );
                                    self.collect_view_call(
                                        view_name,
                                        expr_call.args.clone(),
                                        expr_call.paren_token.clone(),
                                        Some(update_call),
                                        item_counter,
                                        errors,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                }
                self.collect_text_node(None, stage_picker, item_counter, Expr::MethodCall(mcall));
            }
            Expr::Path(expr_path) => {
                self.collect_text_node(None, stage_picker, item_counter, Expr::Path(expr_path));
            }
            Expr::Reference(expr_ref) => {
                self.collect_text_node(None, stage_picker, item_counter, Expr::Reference(expr_ref));
            }
            Expr::Unary(expr_unary) => {
                self.collect_text_node(None, stage_picker, item_counter, Expr::Unary(expr_unary));
            }
            other_expr => {
                let name = get_expr_name(&other_expr);
                errors.error_at2(
                    other_expr.span(),
                    "Expected one of HTML element, view or text. Found ",
                    name,
                );
            }
        }
    }

    fn collect_from_expr_call(
        &mut self,
        at_root: bool,
        expr_call: syn::ExprCall,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) {
        // single_ident(...) like div(..), button(...)
        // SingleIdent(...) like ViewName(...)

        let ExprCall {
            func,
            paren_token,
            args,
            ..
        } = expr_call;
        match expr_as_ident(
            *func,
            "Expected HTML tags (div, input...), ViewName, or spair_list",
        ) {
            Ok(ident) => {
                if ident == "text" {
                    self.collect_text_list(&ident, args, stage_picker, item_counter, errors);
                } else if ident == "spair_list" {
                    self.collect_list(
                        at_root,
                        &ident,
                        paren_token,
                        args,
                        stage_picker,
                        item_counter,
                        errors,
                    );
                } else if ident == "spair_comp_ref" {
                    if args.len() != 1 {
                        errors.error_at(args.span(), "Expected exactly 1 arg");
                        return;
                    }
                    let comp_ref = CompRef::new(
                        ident,
                        args.into_pairs().next().unwrap().into_value(),
                        stage_picker,
                        item_counter,
                    );
                    self.items.push(Item::CompRef(comp_ref));
                } else {
                    let ident_in_string = ident.to_string();
                    let first_letter_is_uppercase = is_first_letter_uppercase(&ident_in_string);
                    if first_letter_is_uppercase {
                        self.collect_view_call(
                            &ident,
                            args,
                            paren_token,
                            None,
                            item_counter,
                            errors,
                        );
                    } else {
                        let element = Element::new(
                            at_root,
                            ident.clone(),
                            args,
                            stage_picker,
                            item_counter,
                            errors,
                        );
                        self.items.push(Item::Element(element));
                    }
                }
            }
            Err(e) => errors.combine(e),
        }
    }

    fn collect_list(
        &mut self,
        at_root: bool,
        spair_list_keyword: &Ident,
        paren_token: syn::token::Paren,
        args: syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) {
        if let Some(list) = List::new(
            at_root,
            spair_list_keyword.clone(),
            paren_token,
            args,
            stage_picker,
            item_counter,
            errors,
        ) {
            self.items.push(Item::List(list));
        };
    }

    fn collect_text_list(
        &mut self,
        text_ident: &Ident,
        args: syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) {
        for expr in args.into_iter() {
            match expr {
                Expr::Lit(expr_lit) => {
                    self.collect_literal_text_node(
                        Some(text_ident.clone()),
                        item_counter,
                        errors,
                        expr_lit,
                    );
                }
                other_expr => {
                    self.collect_text_node(
                        Some(text_ident.clone()),
                        stage_picker,
                        item_counter,
                        other_expr,
                    );
                }
            };
        }
    }

    fn collect_text_node(
        &mut self,
        text_ident: Option<Ident>,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        expr_as_a_text_node: Expr,
    ) {
        self.set_last_text_next_is_a_text_to_true();
        let text =
            Text::with_non_expr_lit(text_ident, expr_as_a_text_node, item_counter, stage_picker);
        self.items.push(Item::Text(text));
    }

    fn collect_literal_text_node(
        &mut self,
        text_ident: Option<Ident>,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
        expr_lit: syn::PatLit,
    ) {
        self.set_last_text_next_is_a_text_to_true();
        if let Some(text_value) = get_static_string(&expr_lit, errors) {
            let lit = Text::with_expr_lit(text_ident, expr_lit, text_value, item_counter);
            self.items.push(Item::Text(lit));
        }
    }

    fn set_last_text_next_is_a_text_to_true(&mut self) {
        if let Some(Item::Text(last_text)) = self.items.last_mut() {
            last_text.set_next_is_a_text();
        }
    }

    fn collect_view_call(
        &mut self,
        view_name: &Ident,
        args: syn::punctuated::Punctuated<Expr, syn::token::Comma>,
        paren_token: syn::token::Paren,
        update_call: Option<ViewFnCall>,
        item_counter: &mut ItemCounter,
        _errors: &mut MultiErrors,
    ) {
        let fn_name = Ident::new("create", view_name.span());
        let create_call = ViewFnCall::new(fn_name, paren_token, args);
        let view = View::new(view_name.clone(), create_call, update_call, item_counter);
        self.items.push(Item::View(view));
    }

    pub fn validate_html(&self, errors: &mut MultiErrors) {
        for child in self.items.iter() {
            child.check_html_multi_errors(errors);
        }
    }

    pub fn prepare_items_for_generating_code(&mut self) {
        for item in self.items.iter_mut() {
            item.prepare_items_for_generating_code(false)
        }
    }

    pub fn generate_view_state_struct_fields(&self, sub_mod: &SubMod) -> TokenStream {
        self.items
            .iter()
            .map(|v| v.generate_view_state_struct_fields(sub_mod))
            .collect()
    }

    pub fn generate_view_states_for_matches_and_lists(&self) -> TokenStream {
        self.items
            .iter()
            .map(|v| v.generate_view_states_for_matches_and_lists())
            .collect()
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        for item in self.items.iter() {
            item.generate_html_string(html_string);
        }
    }

    pub fn generate_fn_create(&self, sub_mod: &SubMod, last_node: &mut LastNode) -> TokenStream {
        self.items
            .iter()
            .map(|v| {
                let code = v.generate_fn_create(sub_mod, last_node);
                last_node.previous = Some(v.spair_ident_to_get_next_node()).cloned();
                code
            })
            .collect()
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        self.items
            .iter()
            .map(|v| v.generate_fn_create_return_value())
            .collect()
    }

    pub fn generate_fn_update(
        &self,
        sub_mod: &SubMod,
        view_state: &Ident,
        parent: &Ident,
    ) -> TokenStream {
        self.items
            .iter()
            .map(|v| v.generate_fn_update(sub_mod, view_state, parent))
            .collect()
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        self.items
            .iter()
            .map(|v| v.generate_fn_remove_from(parent))
            .collect()
    }

    pub fn into_inner(self) -> Vec<Item> {
        self.items
    }
}

fn is_first_letter_uppercase(ident_in_string: &String) -> bool {
    ident_in_string
        .chars()
        .next()
        .map(|v| v.is_uppercase())
        .unwrap_or(false)
}

fn get_expr_name(other_expr: &Expr) -> &str {
    match other_expr {
        Expr::Array(_) => "Expr::Array",
        Expr::Assign(_) => "Expr::Assign",
        Expr::Async(_) => "Expr::Async",
        Expr::Await(_) => "Expr::Await",
        Expr::Binary(_) => "Expr::Binary",
        Expr::Block(_) => "Expr::Block",
        Expr::Break(_) => "Expr::Break",
        Expr::Call(_) => "Expr::Call",
        Expr::Cast(_) => "Expr::Cast",
        Expr::Closure(_) => "Expr::Closure",
        Expr::Const(_) => "Expr::Const",
        Expr::Continue(_) => "Expr::Continue",
        Expr::Field(_) => "Expr::Field",
        Expr::ForLoop(_) => "Expr::ForLoop",
        Expr::Group(_) => "Expr::Group",
        Expr::If(_) => "Expr::If",
        Expr::Index(_) => "Expr::Index",
        Expr::Infer(_) => "Expr::Infer",
        Expr::Let(_) => "Expr::Let",
        Expr::Lit(_) => "Expr::Lit",
        Expr::Loop(_) => "Expr::Loop",
        Expr::Macro(_) => "Expr::Macro",
        Expr::Match(_) => "Expr::Match",
        Expr::MethodCall(_) => "Expr::MethodCall",
        Expr::Paren(_) => "Expr::Paren",
        Expr::Path(_) => "Expr::Path",
        Expr::Range(_) => "Expr::Range",
        Expr::RawAddr(_) => "Expr::RawAddr",
        Expr::Reference(_) => "Expr::Reference",
        Expr::Repeat(_) => "Expr::Repeat",
        Expr::Return(_) => "Expr::Return",
        Expr::Struct(_) => "Expr::Struct",
        Expr::Try(_) => "Expr::Try",
        Expr::TryBlock(_) => "Expr::TryBlock",
        Expr::Tuple(_) => "Expr::Tuple",
        Expr::Unary(_) => "Expr::Unary",
        Expr::Unsafe(_) => "Expr::Unsafe",
        Expr::Verbatim(_) => "Expr::Verbatim",
        Expr::While(_) => "Expr::While",
        Expr::Yield(_) => "Expr::Yield",
        _ => "Unknown expr",
    }
}

fn get_static_string(expr_lit: &syn::PatLit, errors: &mut MultiErrors) -> Option<String> {
    Some(match &expr_lit.lit {
        syn::Lit::Str(lit_str) => lit_str.value(),
        syn::Lit::Char(lit_char) => lit_char.value().to_string(),
        syn::Lit::Int(lit_int) => lit_int.base10_digits().to_string(),
        syn::Lit::Float(lit_float) => lit_float.base10_digits().to_string(),
        syn::Lit::Bool(lit_bool) => lit_bool.value.to_string(),
        other_expr => {
            errors.error_at(other_expr.span(), "This type of literal is not suppported");
            return None;
        }
    })
}

pub struct ItemCounter {
    namespace: String,
    counter: u32,
}

impl ItemCounter {
    pub fn new(namespace: String) -> Self {
        Self {
            namespace,
            counter: 1,
        }
    }

    fn new_ident(&mut self, name: &str) -> Ident {
        let mut name = name.to_string();
        name.push_str(&self.counter.to_string());
        self.counter += 1;
        Ident::new(&name, Span::call_site())
    }

    pub fn new_ident_sub_mod(&mut self, name: &Ident) -> Ident {
        self.new_ident(&format!("sub_mod_{}", name.to_string().to_lowercase()))
    }

    pub fn new_ident_text(&mut self) -> Ident {
        self.new_ident("_text_")
    }

    pub fn new_ident_element(&mut self, element_name: &str) -> Ident {
        self.new_ident(&format!("_el_{element_name}_"))
    }

    pub fn new_ident_view(&mut self) -> Ident {
        self.new_ident("_view_")
    }

    pub fn new_ident_match(&mut self) -> Ident {
        self.new_ident("_match_")
    }

    pub fn new_ident_list(&mut self) -> Ident {
        self.new_ident("_list_")
    }

    pub fn new_ident_marker(&mut self, prefix: &str) -> Ident {
        self.new_ident(&format!("_{prefix}_marker_"))
    }

    fn with_namespace(&mut self, prefix: &str) -> Ident {
        let mut name = self.namespace.clone();
        name.push_str(prefix);
        name.push_str(&self.counter.to_string());
        self.counter += 1;
        Ident::new(&name, Span::call_site())
    }

    fn new_match_enum(&mut self) -> Ident {
        self.with_namespace("MatchEnum")
    }

    fn new_match_struct(&mut self) -> Ident {
        self.with_namespace("MatchStruct")
    }

    fn new_arm_struct(&mut self) -> Ident {
        self.with_namespace("ArmStruct")
    }

    fn new_arm_variant(&mut self) -> Ident {
        self.with_namespace("ArmVariant")
    }

    fn new_list_struct(&mut self) -> Ident {
        self.with_namespace("List")
    }
}

fn expr_as_ident(expr: Expr, message: &str) -> Result<Ident> {
    if let Expr::Path(expr_path) = &expr {
        if let Ok(ident) = expr_path.path.require_ident() {
            return Ok(ident.clone());
        }
    }
    Err(syn::Error::new(
        expr.span(),
        format!("{message}, found expression type: {}", get_expr_name(&expr)),
    ))
}
