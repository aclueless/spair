use std::ops::Not;

use match_expr::Match;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Paren},
    Expr, ExprCall, ExprClosure, ExprMethodCall, Ident, Pat, Result,
};

use crate::{new_view::expr_has_ref_to, ItemCounter, MultiErrors};

mod match_expr;

const REPLACE_AT_ELEMENT_ID: &str = "replace_at_element_id";
const HREF_WITH_ROUTING: &str = "href_with_routing";
const HREF_STR: &str = "href_str";

const VIEW_EXPRESSION_SYNTAX: &str = "div(
    class = \"class names as a string literal\",
    class_if = (bool_expression, class_name),
    button(on_click = callback, text(\"Click me\"))
    v.ViewName(...).update(...),
    l.ListItemName.CompName(...),
    kl.KeyedListItemName.CompName(...),
    websys.node(),
    match expr {
        Pat1 => {}, // can be empty: no content rendered
        Pat2 => div(..., span(...)), // only allow a html element
        Pat3 => v.ViewName(...), // or a child view as a root of a match arm
    }
)";
const CHILD_VIEW_LIST_COMP_SYNTAX: &str = "Expected one of `v.`, `l.`, 'kl.', or `websys.` as in
v.ViewName(...),
l.ListItemName.CompName(...),
kl.KeyedListItemName.CompName(...),
websys.node(...)";

pub(crate) enum Element {
    Text(Text),
    Html(HtmlElement),
    View(View),
    List(List),
    InlinedList(InlinedList),
    Match(Match),
    #[allow(clippy::enum_variant_names)]
    WsElement(WsElement),
}

pub(crate) struct InlinedList {
    pub(crate) name: Ident,
    stage: Stage,
    partial_list: bool,
    component_type_name: Ident,
    item_type_name: Ident,
    key_items: Option<KeyListItems>,
    context: Expr,
    item_iterator: Expr,
    create_view_closure: ExprClosure,
    update_view_closure: ExprClosure,
    element: HtmlElement,

    view_state_type_name: Ident,
    spair_ident: Ident,
    spair_ident_marker: Ident,
}

pub(crate) struct KeyListItems {
    key_type_name: Ident,
    get_key_closure_ident: Ident,
    get_key_closure: Expr,
}

pub(crate) struct List {
    pub(crate) name: Ident,
    keyed_list: bool,
    stage: Stage,
    partial_list: bool,
    component_type_name: Ident,
    item_type_name: Ident,
    context: Expr,
    item_iterator: Expr,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

pub(crate) struct Text {
    pub(crate) shared_name: Ident,
    stage: Stage,
    value: Expr,
    spair_ident: Ident,
    next_node_is_a_text: bool,
}

struct HtmlElementMeta {
    // if > 0, the element type must be spair::Element, otherwise, spair::WsElement
    spair_element_capacity: usize,
    spair_ident: Ident,
}

pub(crate) struct HtmlElement {
    name: Ident,
    attributes: Vec<Attribute>,
    children: Vec<Element>,
    pub(crate) root_element: bool,
    has_match_element: bool,

    meta: HtmlElementMeta,
}

pub(crate) struct View {
    pub(crate) name: Ident,
    create_view_args: Punctuated<Expr, Comma>,
    update_view_method_name: Option<Ident>,
    update_view_args: Option<Punctuated<Expr, Comma>>,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

pub struct WsElement {
    pub(crate) name: Ident,
    ws_element: Expr,

    spair_ident: Ident,
    spair_ident_marker: Ident,
}

struct Attribute {
    stage: Stage,
    key_rust: Ident,
    key_html: Option<Ident>,
    key_attr_string: String,
    value: Expr,

    spair_store_index: usize,
    is_html_event: bool,
}

#[derive(PartialEq, Eq)]
enum Stage {
    HtmlString(String),
    Creation,
    Update,
}

impl Element {
    // Collecting element stops on the first error. It is a bit difficult to collect all the errors at the same time.
    pub fn with_expr(
        expr: Expr,
        item_counter: &mut ItemCounter,
        update_stage_variables: Option<&[String]>,
    ) -> Result<Vec<Self>> {
        match expr {
            Expr::Call(expr_call) => {
                Self::with_expr_call(expr_call, item_counter, update_stage_variables)
            }
            Expr::MethodCall(mcall) => {
                Self::with_expr_method_call(mcall, item_counter, update_stage_variables)
                    .map(|v| vec![v])
            }
            other_expr => Err(syn::Error::new(other_expr.span(), VIEW_EXPRESSION_SYNTAX)),
        }
    }

    fn with_expr_call(
        expr: ExprCall,
        item_counter: &mut ItemCounter,
        update_stage_variables: Option<&[String]>,
    ) -> Result<Vec<Self>> {
        let span = expr.span();
        let ExprCall {
            func,
            paren_token,
            args,
            ..
        } = expr;
        if let Expr::Path(expr_path) = *func {
            let html_tag = expr_path.path.require_ident()?;
            return if html_tag == "text" {
                text_elements(
                    args,
                    paren_token,
                    html_tag,
                    item_counter,
                    update_stage_variables,
                )
            } else {
                HtmlElement::with_name_n_args(
                    html_tag.clone(),
                    args,
                    item_counter,
                    update_stage_variables,
                )
                .map(|v| vec![Element::Html(v)])
            };
        }
        Err(syn::Error::new(
            span,
            "Expected HTML tags (div, input...), v.ViewName, or c.ComponentName",
        ))
    }

    fn with_expr_method_call(
        expr: ExprMethodCall,
        item_counter: &mut ItemCounter,
        update_stage_variables: Option<&[String]>,
    ) -> Result<Self> {
        let span = expr.span();
        let ExprMethodCall {
            receiver,
            method,
            paren_token,
            mut args,
            ..
        } = expr;
        match *receiver {
            Expr::Field(expr_field) => list(
                expr_field,
                method,
                paren_token,
                args,
                item_counter,
                update_stage_variables,
            ),
            Expr::Path(expr_path) => {
                let name = expr_path.path.require_ident()?;
                if name == "v" {
                    Ok(Element::View(View {
                        name: method,
                        create_view_args: args,
                        update_view_method_name: None,
                        update_view_args: None,
                        spair_ident: item_counter.new_ident_view(),
                        spair_ident_marker: item_counter.new_ident("_view_marker"),
                    }))
                } else if name == "ws" {
                    if method != "element" {
                        return Err(syn::Error::new(
                            method.span(),
                            "Expected `element` after a `ws.`",
                        ));
                    }
                    let Some(ws_element) = args.pop() else {
                        return Err(syn::Error::new(method.span(), "Expected a WsElement"));
                    };
                    if args.is_empty().not() {
                        return Err(syn::Error::new(
                            args.iter().nth(1).unwrap_or(ws_element.value()).span(),
                            "Expected exactly one arg for the ws.element. This is the second arg.",
                        ));
                    }
                    Ok(Element::WsElement(WsElement {
                        name: name.clone(),
                        ws_element: ws_element.into_value(),
                        spair_ident: item_counter.new_ident("_ws_element"),
                        spair_ident_marker: item_counter.new_ident("_ws_element_marker"),
                    }))
                } else {
                    Err(syn::Error::new(name.span(), CHILD_VIEW_LIST_COMP_SYNTAX))
                }
            }
            Expr::MethodCall(first_call) => {
                Element::with_double_method_calls(first_call, method, args, item_counter)
            }
            x => Err(syn::Error::new(
                span,
                format!("{} Found {}", CHILD_VIEW_LIST_COMP_SYNTAX, expr_name(&x)),
            )),
        }
    }

    fn span_to_report_error_on_attribute_after_child_node(&self) -> Span {
        match self {
            Element::Html(html_element) => html_element.name.span(),
            Element::Text(text) => text.shared_name.span(),
            Element::View(view) => view.name.span(),
            Element::List(list) => list.name.span(),
            Element::InlinedList(list) => list.name.span(),
            Element::Match(m) => m.match_keyword.span(),
            Element::WsElement(component) => component.name.span(),
        }
    }

    pub fn check_html_multi_errors(&self, errors: &mut MultiErrors) {
        match self {
            Element::Text(_text) => {}
            Element::Html(html_element) => html_element.check_html_multi_errors(errors),
            Element::View(_view) => {}
            Element::List(_list) => {}
            Element::InlinedList(_list) => {}
            Element::Match(m) => m.check_html_multi_errors(errors),
            Element::WsElement(_child_comp) => {}
        }
    }

    fn append_html_string(&self, html_string: &mut String) {
        match self {
            Element::Text(text) => {
                if let Stage::HtmlString(text_value) = &text.stage {
                    html_string.push_str(text_value);
                } else {
                    html_string.push_str("&nbsp;");
                }
            }
            Element::Html(html_element) => html_element.append_html_string(html_string),
            Element::View(_view) => {
                html_string.push_str("<!--view-->");
            }
            Element::List(list) => {
                if list.partial_list {
                    html_string.push_str("<!--plist-->");
                }
            }
            Element::InlinedList(list) => {
                if list.partial_list {
                    html_string.push_str("<!--iplist-->");
                }
            }
            Element::Match(m) => {
                if m.parent_has_only_one_child.not() {
                    html_string.push_str("<!--mi-->")
                }
            }
            Element::WsElement(_child_comp) => html_string.push_str("<!--wse-->"),
        }
    }

    pub(crate) fn prepare_items_for_generating_code(&mut self, parent_has_only_one_child: bool) {
        match self {
            Element::Text(_) => {}
            Element::Html(html_element) => html_element.prepare_items_for_generating_code(),
            Element::View(_) => {}
            Element::List(list) => list.partial_list = parent_has_only_one_child.not(),
            Element::InlinedList(list) => list.partial_list = parent_has_only_one_child.not(),
            Element::Match(m) => m.prepare_items_for_generating_code(parent_has_only_one_child),
            Element::WsElement(_) => {}
        }
    }

    pub(crate) fn generate_view_state_struct_fields(&self) -> TokenStream {
        match self {
            Element::Text(text) => text.generate_view_state_struct_fields(),
            Element::Html(html_element) => html_element.generate_view_state_struct_fields(),
            Element::View(view) => view.generate_view_state_struct_fields(),
            Element::List(list) => list.generate_view_state_struct_fields(),
            Element::InlinedList(list) => list.generate_view_state_struct_fields(),
            Element::Match(m) => m.generate_view_state_struct_fields(),
            Element::WsElement(cr) => cr.generate_view_state_struct_fields(),
        }
    }

    fn generate_match_view_state_types_n_struct_fields(
        &self,
        inner_types: &mut Vec<TokenStream>,
    ) -> TokenStream {
        match self {
            Element::Text(text) => text.generate_match_view_state_types_n_struct_fields(),
            Element::Html(html_element) => {
                html_element.generate_match_view_state_types_n_struct_fields(inner_types)
            }
            Element::View(view) => view.generate_view_state_struct_fields(),
            Element::List(list) => list.generate_view_state_struct_fields(),
            Element::InlinedList(list) => list.generate_view_state_struct_fields(),
            Element::Match(m) => m.generate_match_view_state_types_n_struct_fields(inner_types),
            Element::WsElement(cr) => cr.generate_view_state_struct_fields(),
        }
    }

    fn generate_get_root_ws_element_for_match_arm(&self, view_state: &Ident) -> TokenStream {
        match self {
            Element::Text(_) => quote! {},
            Element::Html(html_element) => {
                let ident = &html_element.meta.spair_ident;
                quote! {Some(&#view_state.#ident)}
            }
            Element::View(view) => {
                let ident = &view.spair_ident;
                quote! {Some(#view_state.#ident.root_element())}
            }
            Element::List(_list) => quote! {None},
            Element::InlinedList(_list) => quote! {None},
            Element::Match(m) => {
                let ident = &m.spair_ident;
                quote! {
                    #view_state.#ident.root_element()
                }
            }
            Element::WsElement(cr) => {
                let ws_element = &cr.spair_ident;
                quote! {
                    Some(&#view_state.#ws_element)
                }
            }
        }
    }

    fn generate_remove_child_b4_changing_to_other_match_arm(
        &self,
        view_state: &Ident,
    ) -> TokenStream {
        match self {
            Element::Text(_) => quote! {},
            Element::Html(html_element) => {
                let ident = &html_element.meta.spair_ident;
                quote! {parent.remove_child(&#view_state.#ident);}
            }
            Element::View(view) => {
                let ident = &view.spair_ident;
                quote! {parent.remove_child(#view_state.#ident.root_element());}
            }
            Element::List(list) => {
                let ident = &list.spair_ident;
                // A list may have many children. The list, itself, has to remove them from the parent.
                quote! {#view_state.#ident.remove_from_parent(parent);}
            }
            Element::InlinedList(list) => {
                let ident = &list.spair_ident;
                // A list may have many children. The list, itself, has to remove them from the parent.
                quote! {#view_state.#ident.remove_from_parent(parent);}
            }
            Element::Match(m) => {
                let ident = &m.spair_ident;
                quote! {
                    if let Some(child_element) = #view_state.#ident.root_element() {
                        parent.remove_child(child_element);
                    }
                }
            }
            Element::WsElement(cr) => {
                let ws_element = &cr.spair_ident;
                quote! {
                    parent.remove_child(&view_state.#ws_element);
                }
            }
        }
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        match self {
            Element::Text(text) => text.generate_fields_for_view_state_instance(),
            Element::Html(html_element) => {
                html_element.generate_fields_for_view_state_instance_construction()
            }
            Element::View(view) => view.generate_fields_for_view_state_instance(),
            Element::List(list) => list.generate_fields_for_view_state_instance(),
            Element::InlinedList(list) => list.generate_fields_for_view_state_instance(),
            Element::Match(m) => m.generate_fields_for_view_state_instance(),
            Element::WsElement(cr) => {
                let ident = &cr.spair_ident;
                quote! {#ident,}
            }
        }
    }

    fn generate_code_for_create_view_fn(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        match self {
            Element::Text(text) => {
                text.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::Html(html_element) => {
                html_element.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::View(view) => {
                view.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::List(list) => {
                list.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::InlinedList(list) => {
                list.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::Match(m) => m.generate_code_for_create_view_fn_as_child_node(parent, previous),
            Element::WsElement(cr) => {
                let ident = &cr.spair_ident;
                let marker = &cr.spair_ident_marker;
                let get_marker = if let Some(previous) = previous {
                    quote! {let #marker = #previous.ws_node_ref().next_sibling_ws_node();}
                } else {
                    quote! {let #marker = #parent.ws_node_ref().first_ws_node();}
                };
                let ws_element = &cr.ws_element;
                quote! {
                    #get_marker
                    let #ident: ::spair::WsElement = #ws_element;
                    #parent.insert_new_node_before_a_node(&#ident, Some(&#marker));
                }
            }
        }
    }

    fn spair_ident_to_get_next_node(&self) -> &Ident {
        match self {
            Element::Text(text) => &text.spair_ident,
            Element::Html(html_element) => &html_element.meta.spair_ident,
            Element::View(view) => &view.spair_ident_marker,
            Element::List(list) => &list.spair_ident_marker,
            Element::InlinedList(list) => &list.spair_ident_marker,
            Element::Match(m) => &m.spair_ident_marker,
            Element::WsElement(cr) => &cr.spair_ident_marker,
        }
    }

    fn generate_code_for_update_view_fn(
        &self,
        parent: &Ident,
        view_state_ident: &Ident,
    ) -> TokenStream {
        match self {
            Element::Text(text) => {
                text.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::Html(html_element) => {
                html_element.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::View(view) => {
                view.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::List(list) => {
                list.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::InlinedList(list) => {
                list.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::Match(m) => {
                m.generate_code_for_update_view_fn_as_child_node(parent, view_state_ident)
            }
            Element::WsElement(_) => quote! {},
        }
    }

    pub fn name_or_text_expr_span(&self) -> Span {
        match self {
            Element::Text(text) => text.value.span(),
            Element::Html(html_element) => html_element.name.span(),
            Element::View(view) => view.name.span(),
            Element::List(list) => list.name.span(),
            Element::InlinedList(list) => list.name.span(),
            Element::Match(m) => m.match_keyword.span,
            Element::WsElement(cr) => cr.name.span(),
        }
    }

    fn collect_match_n_inlined_list_view_state_types(&self) -> TokenStream {
        match self {
            Element::Html(html_element) => {
                html_element.collect_match_n_inlined_list_view_state_types()
            }
            Element::Match(m) => m.generate_match_n_inlined_list_view_state_types(),
            Element::InlinedList(il) => il.generate_match_n_inlined_list_view_state_types(),
            _ => quote! {},
        }
    }

    fn with_double_method_calls(
        first_call: ExprMethodCall,
        second_method_name: Ident,
        second_args: Punctuated<Expr, Comma>,
        item_counter: &mut ItemCounter,
    ) -> Result<Element> {
        let ExprMethodCall {
            receiver,
            method,
            args,
            ..
        } = first_call;
        let ep = match *receiver {
            Expr::Path(ep) => ep,
            other_expr => {
                return Err(syn::Error::new(
                    other_expr.span(),
                    format!(
                        "{CHILD_VIEW_LIST_COMP_SYNTAX} Found {}",
                        expr_name(&other_expr)
                    ),
                ));
            }
        };
        let keyword = ep.path.require_ident()?;
        if keyword == "v" {
            Ok(Element::View(View {
                name: method,
                create_view_args: args,
                update_view_method_name: Some(second_method_name),
                update_view_args: Some(second_args),
                spair_ident: item_counter.new_ident_view(),
                spair_ident_marker: item_counter.new_ident("_view_marker"),
            }))
        } else {
            Err(syn::Error::new(keyword.span(), CHILD_VIEW_LIST_COMP_SYNTAX))
        }
    }
}

fn list(
    expr_field: syn::ExprField,
    component_type_name: Ident,
    paren: Paren,
    args: Punctuated<Expr, Comma>,
    item_counter: &mut ItemCounter,
    update_stage_variables: Option<&[String]>,
) -> Result<Element> {
    match args.len() {
        2 => list_via_trait(
            expr_field,
            component_type_name,
            args,
            item_counter,
            update_stage_variables,
        ),
        5 => inlined_list(
            expr_field,
            component_type_name,
            args,
            item_counter,
            update_stage_variables,
        ),
        // 6 => inlined_keyed_list(
        //     expr_field,
        //     component_type_name,
        //     paren,
        //     args,
        //     item_counter,
        //     update_stage_variables,
        // ),
        _ => Err(syn::Error::new(
            paren.span.span(),
            "Expected one of:
            `(context, items_iterator)` - for a list via trait
            `(context, items_iterator, create_view_closure, update_view_closure, view_element)` - for an inlined non-keyed list
            `not implemented yet: (context, items_iterator, get_key_closure, create_view_closure, update_view_closure, view_element)` - for an inlined keyed list
            ",
        )),
    }
}

fn list_via_trait(
    expr_field: syn::ExprField,
    component_type_name: Ident,
    args: Punctuated<Expr, Comma>,
    item_counter: &mut ItemCounter,
    update_stage_variables: Option<&[String]>,
) -> Result<Element> {
    let mut args = args.into_pairs();
    let message_for_l_or_kl =
        "Expected a keyword `l` or `kl` (which is short for `list` or `keyed_list`)";
    let name = expr_as_ident(*expr_field.base, message_for_l_or_kl)?;
    if name != "l" && name != "kl" {
        return Err(syn::Error::new(name.span(), message_for_l_or_kl));
    }
    let keyed_item_type_name = match expr_field.member {
        syn::Member::Named(ident) => ident,
        syn::Member::Unnamed(index) => {
            return Err(syn::Error::new(index.span(), "Expected KeyedItemName`"));
        }
    };
    let context = args.next().unwrap().into_value();
    let keyed_item_iter = args.next().unwrap().into_value();
    Ok(Element::List(List {
        keyed_list: name == "kl",
        name,
        stage: is_expr_in_create_or_update_stage(&keyed_item_iter, update_stage_variables),
        partial_list: false,
        component_type_name,
        item_type_name: keyed_item_type_name,
        context,
        item_iterator: keyed_item_iter,
        spair_ident: item_counter.new_ident("_list"),
        spair_ident_marker: item_counter.new_ident("_list_end_flag"),
    }))
}

fn inlined_list(
    expr_field: syn::ExprField,
    component_type_name: Ident,
    args: Punctuated<Expr, Comma>,
    item_counter: &mut ItemCounter,
    update_stage_variables: Option<&[String]>,
) -> Result<Element> {
    let mut args = args.into_pairs();
    let message_for_lwa = "Expected a keyword `lwa` (which is short for `list with anotation`)";
    let name = expr_as_ident(*expr_field.base, message_for_lwa)?;
    if name != "lwa" {
        return Err(syn::Error::new(name.span(), message_for_lwa));
    }
    let keyed_item_type_name = match expr_field.member {
        syn::Member::Named(ident) => ident,
        syn::Member::Unnamed(index) => {
            return Err(syn::Error::new(index.span(), "Expected KeyedItemName`"));
        }
    };
    let context = args.next().unwrap().into_value();
    let keyed_item_iter = args.next().unwrap().into_value();
    let create_view_closure = args.next().unwrap().into_value();
    let update_view_closure = args.next().unwrap().into_value();
    let element_expr = args.next().unwrap().into_value();
    let element = Element::with_expr(element_expr, item_counter, update_stage_variables)?.remove(0);
    let Element::Html(mut html) = element else {
        return Err(syn::Error::new(
            element.name_or_text_expr_span(),
            "Expected an html element here. No text, view, child component, list... allowed",
        ));
    };
    html.root_element = true;

    let Expr::Closure(create_view_closure) = create_view_closure else {
        return Err(syn::Error::new(
            create_view_closure.span(),
            "Expected a closure here",
        ));
    };

    let Expr::Closure(update_view_closure) = update_view_closure else {
        return Err(syn::Error::new(
            update_view_closure.span(),
            "Expected a closure here",
        ));
    };

    Ok(Element::InlinedList(InlinedList {
        name,
        stage: is_expr_in_create_or_update_stage(&keyed_item_iter, update_stage_variables),
        partial_list: false,
        component_type_name,
        item_type_name: keyed_item_type_name,
        key_items: None,
        context,
        item_iterator: keyed_item_iter,
        spair_ident: item_counter.new_ident("_ilist"),
        spair_ident_marker: item_counter.new_ident("_ilist_end_flag"),
        create_view_closure,
        update_view_closure,
        element: html,
        view_state_type_name: item_counter.new_ident("_InlinedListViewState"),
    }))
}

fn text_elements(
    args: Punctuated<Expr, Comma>,
    paren: Paren,
    html_tag: &Ident,
    item_counter: &mut ItemCounter,
    update_stage_variables: Option<&[String]>,
) -> Result<Vec<Element>> {
    if args.is_empty() {
        return Err(syn::Error::new(paren.span.span(), "Empty text?"));
    }
    let mut text_nodes = Vec::new();
    let text_node_count = args.len();
    for (index, expr) in args.into_iter().enumerate() {
        let this_is_the_last_text = index + 1 == text_node_count;
        let text_node = match expr {
            Expr::Lit(expr_lit) => {
                let text_value = get_static_string(&expr_lit)?;
                Text {
                    shared_name: html_tag.clone(),
                    stage: Stage::HtmlString(text_value),
                    value: Expr::Lit(expr_lit),
                    spair_ident: item_counter.new_ident_text(),
                    next_node_is_a_text: this_is_the_last_text.not(),
                }
            }
            other_expr => Text {
                shared_name: html_tag.clone(),
                stage: is_expr_in_create_or_update_stage(&other_expr, update_stage_variables),
                value: other_expr,
                spair_ident: item_counter.new_ident_text(),
                next_node_is_a_text: this_is_the_last_text.not(),
            },
        };
        text_nodes.push(text_node);
    }
    Ok(text_nodes.into_iter().map(Element::Text).collect())
}

impl Text {
    fn generate_view_state_struct_fields(&self) -> TokenStream {
        if matches!(self.stage, Stage::Update) {
            let ident = &self.spair_ident;
            quote! {#ident: ::spair::Text,}
        } else {
            quote! {}
        }
    }

    fn generate_match_view_state_types_n_struct_fields(&self) -> TokenStream {
        if matches!(self.stage, Stage::HtmlString(_)) {
            return quote! {};
        }
        let ident = &self.spair_ident;
        quote! {#ident: ::spair::Text,}
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        if matches!(self.stage, Stage::Update) {
            let ident = &self.spair_ident;
            quote! {#ident,}
        } else {
            quote! {}
        }
    }

    fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let text_node = &self.spair_ident;
        let offset = match &self.stage {
            Stage::HtmlString(s) => s.chars().count() as u32,
            Stage::Creation => 1,
            Stage::Update => 1,
        };
        let text = &self.value;
        let get_text_node = |first_text_method_name, next_text_method_name| {
            let first_text_method_name = Ident::new(first_text_method_name, Span::call_site());
            let next_text_method_name = Ident::new(next_text_method_name, Span::call_site());
            let get_text_node = match previous {
                None => {
                    quote! { let #text_node = #parent.ws_node_ref().#first_text_method_name(); }
                }
                Some(previous) => {
                    quote! { let #text_node = #previous.ws_node_ref().#next_text_method_name(); }
                }
            };
            if self.next_node_is_a_text {
                quote! {
                    #get_text_node
                    #text_node.split_text(#offset);
                }
            } else {
                get_text_node
            }
        };
        match self.stage {
            Stage::Creation => {
                let get_text_node = get_text_node("first_ws_text", "next_sibling_ws_text");
                quote! {
                    #get_text_node
                    #text_node.set_text(#text);
                }
            }
            Stage::Update => {
                let get_text_node = get_text_node("first_text", "next_sibling_text");
                quote! {
                    #get_text_node
                }
            }
            Stage::HtmlString(_) => {
                let get_text_node = get_text_node("first_ws_text", "next_sibling_ws_text");
                quote! {
                    #get_text_node
                }
            }
        }
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        view_state_ident: &Ident,
    ) -> TokenStream {
        let text_node = &self.spair_ident;
        let text = &self.value;
        if let Stage::Update = self.stage {
            quote! {
                #view_state_ident.#text_node.update(#text);
            }
        } else {
            quote! {}
        }
    }
}

impl HtmlElement {
    fn with_name_n_args(
        name: Ident,
        args: Punctuated<Expr, syn::token::Comma>,
        item_counter: &mut ItemCounter,
        update_stage_variables: Option<&[String]>,
    ) -> Result<HtmlElement> {
        let element_name = name.to_string();
        let spair_ident = item_counter.new_ident_element();
        let mut attributes = Vec::new();
        let mut children: Vec<Element> = Vec::new();
        for expr in args.into_iter() {
            match expr {
                Expr::Assign(expr_assign) => {
                    if let Some(element) = children.last() {
                        return Err(syn::Error::new(
                            element.span_to_report_error_on_attribute_after_child_node(),
                            "An attribute can not appear after a text or child node",
                        ));
                    }
                    let attribute = Attribute::with_expr_assign(
                        expr_assign,
                        &element_name,
                        update_stage_variables,
                    )?;
                    if attribute.is_html_event && attribute.stage == Stage::Creation {
                        attributes.insert(0, attribute);
                    } else {
                        attributes.push(attribute);
                    }
                }
                Expr::Call(expr_call) => {
                    let vec =
                        Element::with_expr_call(expr_call, item_counter, update_stage_variables)?;
                    if let Some(Element::Text(_)) = vec.first() {
                        if let Some(Element::Text(last)) = children.last_mut() {
                            last.next_node_is_a_text = true;
                        }
                    }
                    children.extend(vec);
                }
                Expr::Match(expr_match) => {
                    children.push(
                        Match::with_expr_match(expr_match, item_counter).map(Element::Match)?,
                    );
                }
                Expr::MethodCall(mcall) => {
                    let item = Element::with_expr_method_call(
                        mcall,
                        item_counter,
                        update_stage_variables,
                    )?;
                    children.push(item);
                }
                other_expr => {
                    return Err(syn::Error::new(
                        other_expr.span(),
                        format!(
                            "Expected {VIEW_EXPRESSION_SYNTAX}, found {}",
                            expr_name(&other_expr)
                        ),
                    ));
                }
            }
        }
        Ok(HtmlElement {
            name,
            attributes,
            children,
            root_element: false,
            has_match_element: false,
            meta: HtmlElementMeta {
                spair_element_capacity: 0,
                spair_ident,
            },
        })
    }

    fn count_spair_element_capacity(&mut self) {
        let mut store_index = 0;
        for attribute in self.attributes.iter_mut().filter(|attribute| {
            attribute.is_html_event || matches!(&attribute.stage, Stage::Update)
        }) {
            attribute.spair_store_index = store_index;
            store_index += 1;
        }

        self.meta.spair_element_capacity = store_index;
    }

    pub fn validate_html(&self) -> Result<()> {
        let mut errors = MultiErrors::default();
        self.check_html_multi_errors(&mut errors);
        errors.report_error()
    }

    fn check_html_multi_errors(&self, errors: &mut MultiErrors) {
        self.check_html_tag(errors);
        for attribute in self.attributes.iter() {
            attribute.check_html(&self.name.to_string(), errors);
        }
        for child in self.children.iter() {
            child.check_html_multi_errors(errors);
        }
    }

    fn check_html_tag(&self, errors: &mut MultiErrors) {
        match self.name.to_string().as_str() {
            "body" | 
            "address" |
            "article" |
            "aside" |
            "footer" |
            "header" |
            "h1" |
            "h2" |
            "h3" |
            "h4" |
            "h5" |
            "h6" |
            "hgroup" |
            "main" |
            "nav" |
            "section" |
            "search" |
            "blockquote" |
            "dd" |
            "div" |
            "dl" |
            "dt" |
            "figcaption" |
            "figure" |
            "hr" |
            "li" |
            "menu" |
            "ol" |
            "p" |
            "pre" |
            "ul" |
            "a" |
            "abbr" |
            "b" |
            "bdi" |
            "bdo" |
            "br" |
            "cite" |
            "code" |
            "data" |
            "dfn" |
            "em" |
            "i" |
            "kbd" |
            "mark" |
            "g" |
            "rp" |
            "rt" |
            "ruby" |
            "s" |
            "samp" |
            "small" |
            "span" |
            "strong" |
            "sub" |
            "sup" |
            "time" |
            "u" |
            "var" |
            "wbr" |
            "area" |
            "audio" |
            "img" |
            "map" |
            "track" |
            "video" |
            "embed" |
            "fencedframe" |
            "iframe" |
            "object" |
            "picture" |
            "portal" |
            "source" |
            "svg" |
            "math" |
            "canvas" |
            "noscript" |
            "script" |
            "del" |
            "ins" |
            "caption" |
            "col" |
            "colgroup" |
            "table" |
            "tbody" |
            "td" |
            "tfoot" |
            "th" |
            "thead" |
            "tr" |
            "button" |
            "datalist" |
            "fieldset" |
            "form" |
            "input" |
            "label" |
            "legend" |
            "meter" |
            "optgroup" |
            "option" |
            "output" |
            "progress" |
            "select" |
            "textarea" |
            "details" |
            "dialog" |
            "summary" 
            // "slot" |
            // "template" |
            => {}
            _ => errors.add(self.name.span(), "unknown html tag"),
        }
    }

    pub(crate) fn construct_html_string(&self) -> String {
        let mut html_string = String::new();
        self.append_html_string(&mut html_string);
        html_string
    }

    fn append_html_string(&self, html_string: &mut String) {
        let html_tag = self.name.to_string();
        let (open_closing, close_1, close_2, close_3) = match html_tag.as_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "source" | "track" | "wbr" => (" />", "", "", ""),
            html_tag => (">", "</", html_tag, ">"),
        };
        html_string.push('<');
        html_string.push_str(&html_tag);
        self.append_html_string_attributes(html_string);
        html_string.push_str(open_closing);
        self.append_html_string_children(html_string);
        html_string.push_str(close_1);
        html_string.push_str(close_2);
        html_string.push_str(close_3);
    }

    fn append_html_string_attributes(&self, html_string: &mut String) {
        for attribute in self.attributes.iter() {
            attribute.construct_html_string(html_string);
        }
    }

    fn append_html_string_children(&self, html_string: &mut String) {
        for element in self.children.iter() {
            element.append_html_string(html_string);
        }
    }

    pub(crate) fn prepare_items_for_generating_code(&mut self) {
        let me_has_only_one_child = self.children.len() == 1;
        let mut has_match_element = false;
        for element in self.children.iter_mut() {
            element.prepare_items_for_generating_code(me_has_only_one_child);
            has_match_element = matches!(element, Element::Match(_));
        }
        self.has_match_element = has_match_element;
        self.count_spair_element_capacity();
    }

    fn generate_view_state_struct_field(&self) -> TokenStream {
        let ident = &self.meta.spair_ident;
        if self.root_element || self.meta.spair_element_capacity > 0 {
            quote! {#ident: ::spair::Element, }
        } else if self.has_match_element {
            quote! {#ident: ::spair::WsElement, }
        } else {
            quote! {}
        }
    }

    pub fn generate_view_state_struct_fields(&self) -> TokenStream {
        let self_element = self.generate_view_state_struct_field();
        let children: TokenStream = self
            .children
            .iter()
            .map(|v| v.generate_view_state_struct_fields())
            .collect();
        quote! {
            #self_element
            #children
        }
    }

    fn generate_match_view_state_types_n_struct_fields(
        &self,
        inner_types: &mut Vec<TokenStream>,
    ) -> TokenStream {
        let self_element = self.generate_view_state_struct_field();
        let children: TokenStream = self
            .children
            .iter()
            .map(|v| v.generate_match_view_state_types_n_struct_fields(inner_types))
            .collect();
        quote! {
            #self_element
            #children
        }
    }

    fn generate_fields_for_view_state_instance_construction(&self) -> TokenStream {
        let ident = &self.meta.spair_ident;
        let self_element = if self.root_element
            || self.has_match_element
            || self.meta.spair_element_capacity > 0
        {
            quote! {#ident,}
        } else {
            quote! {}
        };
        let view_state_fields: TokenStream = self
            .children
            .iter()
            .map(|v| v.generate_fields_for_view_state_instance())
            .collect();
        quote! {
            #self_element
            #view_state_fields
        }
    }

    fn generate_view_state_instance_construction(
        &self,
        view_state_struct_name: &Ident,
    ) -> TokenStream {
        let fields = self.generate_fields_for_view_state_instance_construction();
        quote! {
            #view_state_struct_name{
                #fields
            }
        }
    }

    fn generate_view_state_instance_construction_with_extra_field(
        &self,
        view_state_struct_name: &Ident,
        extra_field: TokenStream,
    ) -> TokenStream {
        let fields = self.generate_fields_for_view_state_instance_construction();
        quote! {
            #view_state_struct_name{
                #extra_field
                #fields
            }
        }
    }

    pub(crate) fn root_element_type(&self) -> Ident {
        Ident::new("Element", Span::call_site())
    }

    pub(crate) fn root_element_ident(&self) -> &Ident {
        &self.meta.spair_ident
    }

    pub fn generate_fn_body_for_create_view_fn_of_a_view(
        &self,
        view_state_struct_name: &Ident,
    ) -> TokenStream {
        let first_part = self.generate_code_for_create_view_fn();
        let construct_view_state_instance =
            self.generate_view_state_instance_construction(view_state_struct_name);
        quote! {
            #first_part
            #construct_view_state_instance
        }
    }

    pub fn generate_code_for_create_view_fn_of_a_component(
        &self,
        view_state_struct_name: &Ident,
    ) -> TokenStream {
        let root_element = &self.meta.spair_ident;
        let first_part = self.generate_code_for_create_view_fn();
        let construct_view_state_instance =
            self.generate_view_state_instance_construction(view_state_struct_name);
        quote! {
            #first_part
            (#root_element.ws_element().clone(), #construct_view_state_instance)
        }
    }

    pub fn generate_code_for_create_view_fn_of_a_keyed_item_view(
        &self,
        view_state_struct_name: &Ident,
        key_for_view_state_instance: TokenStream,
    ) -> TokenStream {
        let root_element = &self.meta.spair_ident;
        let capacity = self.meta.spair_element_capacity;
        let attribute_setting = self.generate_attribute_code_for_create_view_fn();
        let children = self.generate_children_code_for_create_view_fn();
        let construct_view_state_instance = self
            .generate_view_state_instance_construction_with_extra_field(
                view_state_struct_name,
                key_for_view_state_instance,
            );
        quote! {
            let #root_element = _keyed_view_state_template.create_element(#capacity);
            #attribute_setting
            #children
            #construct_view_state_instance
        }
    }

    fn generate_code_for_create_view_fn(&self) -> TokenStream {
        let html_string = self.construct_html_string();
        let root_element = &self.meta.spair_ident;
        let capacity = self.meta.spair_element_capacity;
        let attribute_setting = self.generate_attribute_code_for_create_view_fn();
        let children = self.generate_children_code_for_create_view_fn();
        quote! {
            const HTML_STRING: &str = #html_string;
            let mut #root_element = ::spair::Element::with_html(HTML_STRING, #capacity);
            #attribute_setting
            #children
        }
    }

    fn generate_attribute_code_for_create_view_fn(&self) -> TokenStream {
        let element = &self.meta.spair_ident;
        let element_name = self.name.to_string();

        self.attributes
            .iter()
            .map(|v| v.generate_attribute_code_for_create_view_fn(&element_name, element))
            .collect()
    }

    fn generate_children_code_for_create_view_fn(&self) -> TokenStream {
        let element = &self.meta.spair_ident;
        let mut previous = None;
        self.children
            .iter()
            .map(|v| {
                let code = v.generate_code_for_create_view_fn(element, previous);
                previous = Some(v.spair_ident_to_get_next_node());
                code
            })
            .collect()
    }

    fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let element = &self.meta.spair_ident;
        let get_ws_element = match previous {
            Some(previous) => {
                quote! {let #element = #previous.ws_node_ref().next_sibling_ws_element();}
            }
            None => quote! { let #element = #parent.ws_node_ref().first_ws_element(); },
        };
        let get_element = if self.meta.spair_element_capacity > 0 {
            let capacity = self.meta.spair_element_capacity;
            quote! {
                #get_ws_element
                let mut #element = #element.create_element_with_capacity(#capacity);
            }
        } else {
            get_ws_element
        };
        let set_attributes = self.generate_attribute_code_for_create_view_fn();
        let children = self.generate_children_code_for_create_view_fn();

        quote! {
            #get_element
            #set_attributes
            #children
        }
    }

    pub(crate) fn generate_code_for_update_view_fn(&self, view_state_ident: &Ident) -> TokenStream {
        let attribute_setting = self.generate_attribute_code_for_update_view_fn(view_state_ident);
        let children = self.generate_children_code_for_update_view_fn(view_state_ident);
        quote! {
            #attribute_setting
            #children
        }
    }

    fn generate_attribute_code_for_update_view_fn(&self, view_state_ident: &Ident) -> TokenStream {
        let element = &self.meta.spair_ident;

        let element = quote! {#view_state_ident.#element};
        let element_name = self.name.to_string();
        self.attributes
            .iter()
            .map(|v| v.generate_attribute_code_for_update_view_fn(&element_name, &element))
            .collect()
    }

    fn generate_children_code_for_update_view_fn(&self, view_state_ident: &Ident) -> TokenStream {
        self.children
            .iter()
            .map(|v| v.generate_code_for_update_view_fn(&self.meta.spair_ident, view_state_ident))
            .collect()
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        view_state_ident: &Ident,
    ) -> TokenStream {
        let set_attributes = self.generate_attribute_code_for_update_view_fn(view_state_ident);
        let children = self.generate_children_code_for_update_view_fn(view_state_ident);
        quote! {
            #set_attributes
            #children
        }
    }

    pub(crate) fn collect_match_n_inlined_list_view_state_types(&self) -> TokenStream {
        self.children
            .iter()
            .map(|v| v.collect_match_n_inlined_list_view_state_types())
            .collect()
    }
}

impl Attribute {
    fn with_expr_assign(
        expr: syn::ExprAssign,
        element_name: &str,
        update_stage_variables: Option<&[String]>,
    ) -> Result<Self> {
        let key_rust = expr_as_ident(
            *expr.left,
            "Expected a single identifier as an HTML attribute name",
        )?;
        let key_rust_string = key_rust.to_string();
        let mut is_html_event = false;
        let (key_html, key_html_string) =
            if let Some(key_html_string) = key_rust_string.strip_prefix("on_") {
                is_html_event = is_html_event_name(key_html_string, element_name);
                if !is_html_event {
                    return Err(syn::Error::new(
                        key_rust.span(),
                        format!("Unknown event `{key_html_string}`"),
                    ));
                }
                (
                    Some(Ident::new(key_html_string, Span::call_site())),
                    key_html_string.to_string(),
                )
            } else if let Some(key) = key_rust_string.strip_prefix("r#") {
                (None, key.to_string())
            } else if key_rust_string.starts_with("data_") {
                (None, key_rust_string.replace("_", "-"))
            } else if key_rust_string.starts_with("aria_") {
                (None, key_rust_string.replacen("aria_", "aria-", 1))
            } else {
                (Some(key_rust.clone()), key_rust_string)
            };
        let stage = match &*expr.right {
            Expr::Lit(expr_lit) => {
                let s = get_static_string(expr_lit)?;
                if is_html_event {
                    return Err(syn::Error::new(
                        expr_lit.span(),
                        "An event cannot have a literal value. An event value must be a callback.",
                    ));
                }
                Stage::HtmlString(s)
            }
            other_expr => is_expr_in_create_or_update_stage(other_expr, update_stage_variables),
        };
        let attribute = Attribute {
            stage,
            key_rust,
            key_html,
            key_attr_string: key_html_string,
            value: *expr.right,

            is_html_event,
            spair_store_index: 0,
        };
        Ok(attribute)
    }

    fn check_html(&self, element_name: &str, errors: &mut MultiErrors) {
        if self.is_html_event {
            return;
        }
        if self.key_attr_string.starts_with("data-") {
            return;
        }
        if self.key_attr_string.starts_with("aria-") {
            return;
        }
        match self.key_attr_string.as_str() {
            "class_if" => {
                let message = "`class_if` requires a tuple of 2 expressions as `(boolean_expr, some_class_name)`";
                match &self.value {
                    Expr::Tuple(expr) => {
                        if expr.elems.len() < 2 {
                            errors.add(expr.span(), message);
                        }
                        if let Some(third) = expr.elems.get(2) {
                            errors.add(third.span(), "`class_if` requires exactly 2 expressions");
                        }
                    }
                    other => errors.add(other.span(), message),
                }
            }
            _ => {
                check_html_attribute_name(
                    &self.key_rust,
                    &self.key_attr_string,
                    element_name,
                    errors,
                );
            }
        }
    }

    fn construct_html_string(&self, html_string: &mut String) {
        match self.key_attr_string.as_str() {
            REPLACE_AT_ELEMENT_ID | HREF_WITH_ROUTING => {}
            other_attribute => {
                if let Stage::HtmlString(value) = &self.stage {
                    html_string.push(' ');
                    match other_attribute {
                        HREF_STR => html_string.push_str("href"),
                        other_attribute => html_string.push_str(other_attribute),
                    }
                    html_string.push_str("='");
                    html_string.push_str(value);
                    html_string.push('\'');
                }
            }
        }
    }

    fn generate_attribute_code_for_create_view_fn(
        &self,
        element_name: &str,
        element: &Ident,
    ) -> TokenStream {
        let attribute_value = &self.value;
        if self.key_attr_string == REPLACE_AT_ELEMENT_ID {
            // REPLACE_AT_ELEMENT_ID is a special attribute that always executes in create_view stage to attach the component to DOM
            return quote! {#element.replace_at_element_id(#attribute_value);};
        }
        let is_in_create_mode = matches!(&self.stage, Stage::Creation);
        if is_in_create_mode.not() {
            return if self.key_attr_string == HREF_WITH_ROUTING {
                quote! {#element.add_click_event_to_handle_routing();}
            } else {
                quote! {}
            };
        }
        if self.is_html_event {
            return self.generate_attribute_code_for_event_listener(&quote! {#element});
        }
        match self.key_attr_string.as_str() {
            REPLACE_AT_ELEMENT_ID => {
                unreachable!("Already handle this case before checking for creation mode")
            }
            HREF_STR => quote! {#element.set_str_attribute("href",#attribute_value);},
            HREF_WITH_ROUTING => quote! {
                #element.href_with_routing(#attribute_value);
                #element.add_click_event_to_handle_routing();
            },
            "id" => quote! {#element.set_id(#attribute_value);},
            "class" => quote! {#element.class(#attribute_value);},
            "disabled" => quote! {#element.set_bool_attribute("disabled", #attribute_value);},
            "enabled" => quote! {#element.set_bool_attribute("disabled", !(#attribute_value));},
            "value" => match element_name {
                "select" => quote! {#element.set_select_value(#attribute_value);},
                "input" => quote! {#element.set_input_value(#attribute_value);},
                "textarea" => quote! {#element.set_textarea_value(#attribute_value);},
                "option" => quote! {#element.set_option_value(#attribute_value);},
                _ => {
                    let message = format!("`value` is not an attribute or property of `{element_name}`. Actually, Spair's proc-macros must be update to the report error earlier and at the exact location of the code.");
                    quote! {comple_error(#message);}
                }
            },
            _other_name => {
                let message = format!(
                    "`{}` attribute in create view not implemented yet.",
                    self.key_attr_string
                );
                quote! {compile_error!(#message);}
            }
        }
    }

    fn key(&self) -> &Ident {
        self.key_html.as_ref().unwrap_or(&self.key_rust)
    }

    fn generate_attribute_code_for_event_listener(&self, element: &TokenStream) -> TokenStream {
        let index = self.spair_store_index;
        let attribute_value = &self.value;
        let key = self.key();
        quote! {#element.#key(#index, #attribute_value);}
    }

    fn generate_attribute_code_for_update_view_fn(
        &self,
        element_name: &str,
        element: &TokenStream,
    ) -> TokenStream {
        let is_in_update_mode = matches!(&self.stage, Stage::Update);
        if is_in_update_mode.not() {
            return quote! {};
        }
        let index = self.spair_store_index;
        let attribute_value = &self.value;
        if self.is_html_event {
            return self.generate_attribute_code_for_event_listener(element);
        }
        match self.key_attr_string.as_str() {
            REPLACE_AT_ELEMENT_ID => quote! {},
            HREF_WITH_ROUTING => {
                quote! {#element.href_with_routing_with_index(#index,#attribute_value);}
            }
            "class" => quote! {},
            "class_if" => {
                if let Expr::Tuple(expr) = attribute_value {
                    let condition_expr = &expr.elems[0];
                    let class_name = &expr.elems[1];
                    quote! {
                        #element.class_if_with_index(#index, #condition_expr, #class_name);
                    }
                } else {
                    quote! {}
                }
            }
            "disabled" => {
                quote! {#element.set_bool_attribute_with_index(#index, "disabled", #attribute_value);}
            }
            "enabled" => {
                quote! {#element.set_bool_attribute_with_index(#index, "disabled", !#attribute_value);}
            }
            "value" => match element_name {
                "input" => quote! {#element.set_input_value_with_index(#index, #attribute_value);},
                "textarea" => {
                    quote! {#element.set_textarea_value_with_index(#index, #attribute_value);}
                }
                "select" => {
                    quote! {#element.set_select_value_with_index(#index, #attribute_value);}
                }
                "option" => {
                    quote! {#element.set_option_value_with_index(#index, #attribute_value);}
                }
                _ => {
                    let message = format!("`value` is not an attribute or property of `{element_name}`. Actually, Spair's proc-macros must be update to the report error earlier and at the exact location of the code.");
                    quote! {comple_error(#message);}
                }
            },
            "checked" => quote! {#element.set_input_checked_with_index(#index, #attribute_value);},
            _other_name => {
                let message = format!(
                    "`{}` attribute in update view not implemented yet.",
                    self.key_attr_string
                );
                quote! {compile_error!(#message);}
            }
        }
    }
}

fn is_expr_in_create_or_update_stage(
    expr: &Expr,
    update_stage_variables: Option<&[String]>,
) -> Stage {
    if update_stage_variables
        .map(|update_stage_variables| expr_has_ref_to(expr, update_stage_variables))
        .unwrap_or(true)
    {
        Stage::Update
    } else {
        Stage::Creation
    }
}

fn expr_as_ident(expr: Expr, message: &str) -> Result<Ident> {
    match expr {
        Expr::Path(mut expr_path) if expr_path.path.segments.len() == 1 => {
            Ok(expr_path.path.segments.pop().unwrap().into_value().ident)
        }
        other_expr => Err(syn::Error::new(
            other_expr.span(),
            format!(
                "{message}, found expression type: {}",
                expr_name(&other_expr)
            ),
        )),
    }
}

fn expr_name(expr: &Expr) -> &str {
    match expr {
        Expr::Array(_) => "expr_array",
        Expr::Assign(_) => "expr_assign",
        Expr::Async(_) => "expr_async",
        Expr::Await(_) => "expr_await",
        Expr::Binary(_) => "expr_binary",
        Expr::Block(_) => "expr_block",
        Expr::Break(_) => "expr_break",
        Expr::Call(_) => "expr_call",
        Expr::Cast(_) => "expr_cast",
        Expr::Closure(_) => "expr_closure",
        Expr::Const(_) => "expr_const",
        Expr::Continue(_) => "expr_continue",
        Expr::Field(_) => "expr_field",
        Expr::ForLoop(_) => "expr_for_loop",
        Expr::Group(_) => "expr_group",
        Expr::If(_) => "expr_if",
        Expr::Index(_) => "expr_index",
        Expr::Infer(_) => "expr_infer",
        Expr::Let(_) => "expr_let",
        Expr::Lit(_) => "expr_lit",
        Expr::Loop(_) => "expr_loop",
        Expr::Macro(_) => "expr_macro",
        Expr::Match(_) => "expr_match",
        Expr::MethodCall(_) => "expr_method_call",
        Expr::Paren(_) => "expr_paren",
        Expr::Path(_) => "expr_path",
        Expr::Range(_) => "expr_range",
        Expr::RawAddr(_) => "expr_raw_addr",
        Expr::Reference(_) => "expr_reference",
        Expr::Repeat(_) => "expr_repeat",
        Expr::Return(_) => "expr_return",
        Expr::Struct(_) => "expr_struct",
        Expr::Try(_) => "expr_try",
        Expr::TryBlock(_) => "expr_try_block",
        Expr::Tuple(_) => "expr_tuple",
        Expr::Unary(_) => "expr_unary",
        Expr::Unsafe(_) => "expr_unsafe",
        Expr::Verbatim(_) => "token_stream",
        Expr::While(_) => "expr_while",
        Expr::Yield(_) => "expr_yield",
        _ => "unknown expr type",
    }
}
impl List {
    fn generate_view_state_struct_fields(&self) -> TokenStream {
        let ident = &self.spair_ident;
        let component_type_name = &self.component_type_name;
        let keyed_item_type_name = &self.item_type_name;
        if self.keyed_list {
            quote! {#ident: ::spair::KeyedList<
                    #component_type_name,
                    #keyed_item_type_name,
                    <#keyed_item_type_name as ::spair::KeyedListItemView<#component_type_name>>::Key,
                    <#keyed_item_type_name as ::spair::KeyedListItemView<#component_type_name>>::ViewState,
                >,
            }
        } else {
            quote! {#ident: ::spair::List<
                    #component_type_name,
                    #keyed_item_type_name,
                    <#keyed_item_type_name as ::spair::ListItemView<#component_type_name>>::ViewState,
                >,
            }
        }
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {#ident,}
    }

    fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let ident = &self.spair_ident;
        let marker_ident = &self.spair_ident_marker;
        let end_node = if self.partial_list {
            match previous {
                Some(previous) => quote! {
                    let #marker_ident = #previous.ws_node_ref().next_ws_node();
                    let #parent = #parent.clone();
                },
                None => quote! {
                    let #marker_ident = #parent.ws_node_ref().first_ws_node();
                    let #parent = #parent.clone();
                },
            }
        } else {
            quote! {let #marker_ident = None;}
        };
        let item_type_name = &self.item_type_name;
        let create_list = if self.keyed_list {
            quote! {::spair::KeyedList::new(
                &#parent,
                #marker_ident.clone(),
                #item_type_name::template_string(),
                #item_type_name::get_key,
                #item_type_name::key_from_view_state,
                #item_type_name::create,
                #item_type_name::update,
                #item_type_name::root_element,
            )}
        } else {
            quote! {::spair::List::new(
                &#parent,
                #marker_ident.clone(),
                #item_type_name::template_string(),
                #item_type_name::create,
                #item_type_name::update,
                #item_type_name::root_element,
            )}
        };
        let items_iter = &self.item_iterator;
        let context = &self.context;
        let render_on_creation = if matches!(&self.stage, Stage::Creation) {
            quote! {#ident.update(#items_iter, #context);}
        } else {
            quote! {}
        };
        quote! {
            #end_node
            let #ident = #create_list;
            #render_on_creation
        }
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        view_state_ident: &Ident,
    ) -> TokenStream {
        if matches!(&self.stage, Stage::Update).not() {
            return quote! {};
        }
        let ident = &self.spair_ident;
        let items_iter = &self.item_iterator;
        let context = &self.context;
        quote! {
            #view_state_ident.#ident.update(#items_iter, #context);
        }
    }
}

impl InlinedList {
    fn generate_view_state_struct_fields(&self) -> TokenStream {
        let ident = &self.spair_ident;
        let component_type_name = &self.component_type_name;
        let item_type_name = &self.item_type_name;
        let item_type_name = if item_type_name == "str" {
            quote! {&'static #item_type_name}
        } else {
            quote! {#item_type_name}
        };
        let view_state_type_name = &self.view_state_type_name;
        if let Some(key_items) = self.key_items.as_ref() {
            let key_type_name = &key_items.key_type_name;
            quote! {#ident: ::spair::KeyedList<
                    #component_type_name,
                    #item_type_name,
                    #key_type_name,
                    #view_state_type_name,
                >,
            }
        } else {
            quote! {#ident: ::spair::List<
                    #component_type_name,
                    #item_type_name,
                    #view_state_type_name,
                >,
            }
        }
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {#ident,}
    }

    fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let ident = &self.spair_ident;
        let marker_ident = &self.spair_ident_marker;
        let end_node = if self.partial_list {
            match previous {
                Some(previous) => quote! {
                    let #marker_ident = #previous.ws_node_ref().next_ws_node();
                    let #parent = #parent.clone();
                },
                None => quote! {
                    let #marker_ident = #parent.ws_node_ref().first_ws_node();
                    let #parent = #parent.clone();
                },
            }
        } else {
            quote! {let #marker_ident = None;}
        };
        let view_state_type_name = &self.view_state_type_name;
        let template_string = self.element.construct_html_string();
        let ExprClosure {
            or1_token,
            inputs,
            or2_token,
            output,
            body: let_bindings,
            ..
        } = &self.create_view_closure;
        let fn_body = if let Some(key_items) = self.key_items.as_ref() {
            let get_key_ident = &key_items.get_key_closure_ident;
            let get_key_closure = &key_items.get_key_closure;
            let key = Ident::new("_inlined_keyed_list_item_key_", Span::call_site());
            let key_field = quote! {key: #key,};

            let root_element = &self.element.meta.spair_ident;
            let capacity = self.element.meta.spair_element_capacity;
            let attribute_setting = self.element.generate_attribute_code_for_create_view_fn();
            let children = self.element.generate_children_code_for_create_view_fn();
            let construct_view_state_instance = self
                .element
                .generate_view_state_instance_construction_with_extra_field(
                    view_state_type_name,
                    key_field,
                );

            let item_data = match inputs.first() {
                Some(Pat::Ident(item_data)) => quote! {#item_data},
                _ => quote! {
                    compile_error!("There should be an input for keyed list item data")
                },
            };
            quote! {
                let #get_key_ident = #get_key_closure;
                let #key = Clone::clone(#get_key_ident(#item_data));
                let #root_element = _keyed_view_state_template.create_element(#capacity);
                #attribute_setting
                #children
                #construct_view_state_instance
            }
        } else {
            self.element
                .generate_fn_body_for_create_view_fn_of_a_view(view_state_type_name)
        };
        let create_view_fn = quote! {
            #or1_token _keyed_view_state_template: &::spair::TemplateElement, #inputs #or2_token #output {
                #let_bindings
                #fn_body
            }
        };
        let ExprClosure {
            or1_token,
            inputs,
            or2_token,
            output,
            body: let_bindings,
            ..
        } = &self.update_view_closure;
        let view_state_ident = Ident::new("_inlined_list_item_view_state_", Span::call_site());
        let fn_body = self
            .element
            .generate_code_for_update_view_fn(&view_state_ident);
        let update_view_fn = quote! {
            #or1_token #view_state_ident: &mut #view_state_type_name, #inputs #or2_token #output {
                #let_bindings
                #fn_body
            }
        };
        let create_list = if let Some(key_items) = self.key_items.as_ref() {
            let get_key_ident = &key_items.get_key_closure_ident;
            quote! {::spair::KeyedList::new(
                &#parent,
                #marker_ident.clone(),
                #template_string,
                #get_key_ident,
                #view_state_type_name::get_key,
                #create_view_fn,
                #update_view_fn,
                #view_state_type_name::root_element,
            )}
        } else {
            quote! {::spair::List::new(
                &#parent,
                #marker_ident.clone(),
                #template_string,
                #create_view_fn,
                #update_view_fn,
                #view_state_type_name::root_element,
            )}
        };
        let items_iter = &self.item_iterator;
        let context = &self.context;
        let render_on_creation = if matches!(&self.stage, Stage::Creation) {
            quote! {
                let mut #ident = #ident;
                #ident.update(#items_iter, #context);
            }
        } else {
            quote! {}
        };
        quote! {
            #end_node
            let #ident = #create_list;
            #render_on_creation
        }
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        view_state_ident: &Ident,
    ) -> TokenStream {
        if matches!(&self.stage, Stage::Update).not() {
            return quote! {};
        }
        let ident = &self.spair_ident;
        let items_iter = &self.item_iterator;
        let context = &self.context;
        quote! {
            #view_state_ident.#ident.update(#items_iter, #context);
        }
    }

    fn generate_match_n_inlined_list_view_state_types(&self) -> TokenStream {
        let view_state_type_name = &self.view_state_type_name;
        let fields = self.element.generate_view_state_struct_fields();
        let root_element = &self.element.meta.spair_ident;
        if let Some(key_items) = self.key_items.as_ref() {
            let key_type_name = &key_items.key_type_name;
            quote! {
                struct #view_state_type_name {
                    key: #key_type_name,
                    #fields
                }
                impl #view_state_type_name {
                    fn get_key(&self) -> &#key_type_name {
                        &self.key
                    }
                    fn root_element(&self) -> &::spair::WsElement {
                        &self.#root_element
                    }
                }
            }
        } else {
            quote! {
                struct #view_state_type_name {
                    #fields
                }
                impl #view_state_type_name {
                    fn root_element(&self) -> &::spair::WsElement {
                        &self.#root_element
                    }
                }
            }
        }
    }
}

impl WsElement {
    fn generate_view_state_struct_fields(&self) -> TokenStream {
        let spair_ident = &self.spair_ident;
        quote! {
            #spair_ident: ::spair::WsElement,
        }
    }
}

fn check_html_attribute_name(
    ident: &Ident,
    attribute_name: &str,
    element_name: &str,
    errors: &mut MultiErrors,
) {
    let elements: &[&str] = match attribute_name {
        REPLACE_AT_ELEMENT_ID => return, // spair attribute: there is an element (element A) given in html document (which has `id` given by this attribute). Spair will put this element (created in spair component) in place of the element A.
        "accept" => &["form", "input"],
        "accept_charset" => &["form"],
        "accesskey" => return, // global attribute
        "action" => &["form"],
        "allow" => &["iframe"],
        "alt" => &["area", "img", "input"],
        "as" => &["link"],
        "async" => &["script"],
        "autocapitalize" => return, // global
        "autocomplete" => &["form", "input", "select", "textarea"],
        "autofocus" => return, // global
        "autoplay" => &["audio", "video"],
        "capture" => &["input"],
        "charset" => &["meta"],
        "checked" => &["input"],
        "cite" => &["blockquote", "del", "ins", "q"],
        "class" => return, // global
        "cols" => &["textarea"],
        "colspan" => &["td", "th"],
        "content" => &["meta"],
        "contenteditable" => return, // global
        "controls" => &["audio", "video"],
        "coords" => &["area"],
        "crossorigin" => &["audio", "img", "link", "script", "video"],
        "csp" => &["iframe"],
        "data" => &["object"],
        "datetime" => &["del", "ins", "time"],
        "decoding" => &["img"],
        "default" => &["track"],
        "defer" => &["script"],
        "dir" => return, // global
        "dirname" => &["input", "textarea"],
        "disabled" => &[
            "button", "fieldset", "input", "optgroup", "option", "select", "textarea",
        ],
        "enabled" => &[
            "button", "fieldset", "input", "optgroup", "option", "select", "textarea",
        ],
        "download" => &["a", "area"],
        "draggable" => return, // global
        "enctype" => &["form"],
        "enterkeyhint" => &["textarea", "contenteditable"],
        "for" => &["label", "output"],
        "form" => &[
            "button", "fieldset", "input", "label", "meter", "object", "output", "progress",
            "select", "textarea",
        ],
        "formaction" => &["input", "button"],
        "formenctype" => &["input", "button"],
        "formmethod" => &["input", "button"],
        "formnovalidate" => &["input", "button"],
        "formtarget" => &["input", "button"],
        "headers" => &["td", "th"],
        "height" => &[
            "canvas", "embed", "iframe", "img", "input", "object", "video",
        ],
        "hidden" => return, // global
        "high" => &["meter"],
        "href" => {
            errors.add(
                ident.span(),
                &format!(
                    "Explicitly use `{HREF_WITH_ROUTING}` or `{HREF_STR}` instead of just `href`"
                ),
            );
            return;
        }
        HREF_STR => &["a", "area", "base", "link"],
        HREF_WITH_ROUTING => &["a", "area"], // Will be handled by router
        "hreflang" => &["a", "link"],
        "http-equiv" => &["meta"],
        "id" => return, // global
        "integrity" => &["link", "script"],
        "inputmode" => &["textarea", "contenteditable"],
        "ismap" => &["img"],
        "itemprop" => return, // global
        "kind" => &["track"],
        "label" => &["optgroup", "option", "track"],
        "lang" => return, // global
        "loading" => &["img", "iframe"],
        "list" => &["input"],
        "loop" => &["audio", "video"],
        "low" => &["meter"],
        "max" => &["input", "meter", "progress"],
        "maxlength" => &["input", "textarea"],
        "minlength" => &["input", "textarea"],
        "media" => &["a", "area", "link", "source", "style"],
        "method" => &["form"],
        "min" => &["input", "meter"],
        "multiple" => &["input", "select"],
        "muted" => &["audio", "video"],
        "name" => &[
            "button", "form", "fieldset", "iframe", "input", "object", "output", "select",
            "textarea", "map", "meta", "param",
        ],
        "novalidate" => &["form"],
        "open" => &["details", "dialog"],
        "optimum" => &["meter"],
        "pattern" => &["output"],
        "ping" => &["a", "area"],
        "placeholder" => &["input", "textarea"],
        "playsinline" => &["video"],
        "poster" => &["video"],
        "preload" => &["audio", "video"],
        "readonly" => &["input", "textarea"],
        "referrerpolicy" => &["a", "area", "iframe", "img", "link", "script"],
        "rel" => &["a", "area", "link"],
        "required" => &["input", "select", "textarea"],
        "reversed" => &["ol"],
        "role" => return, // global
        "rows" => &["textarea"],
        "rowspan" => &["td", "th"],
        "sandbox" => &["iframe"],
        "scope" => &["th"],
        "selected" => &["option"],
        "shape" => &["a", "area"],
        "size" => &["input", "select"],
        "sizes" => &["link", "img", "source"],
        "slot" => return, // global
        "span" => &["col", "colgroup"],
        "spellcheck" => return, // global
        "src" => &[
            "audio", "embed", "iframe", "img", "input", "script", "source", "track", "video",
        ],
        "srcdoc" => &["iframe"],
        "srclang" => &["track"],
        "srcset" => &["img", "source"],
        "start" => &["ol"],
        "step" => &["input"],
        "style" => return,      // global
        "tableindex" => return, // global
        "target" => &["a", "area", "base", "form"],
        "title" => return,     // global
        "translate" => return, // global
        "type" => &[
            "button", "input", "embed", "object", "ol", "script", "source", "style", "menu", "link",
        ],
        "usemap" => &["img", "input", "object"],
        "value" => &[
            "button", "data", "input", "li", "meter", "option", "progress", "param",
            "select", // `value` a property of <select>
        ],
        "width" => &[
            "canvas", "embed", "iframe", "img", "input", "object", "video",
        ],
        "wrap" => &["textarea"],
        _ => &[],
    };
    if elements.contains(&element_name).not() {
        errors.add(
            ident.span(),
            &format!("Unknown attribute for `<{element_name}>`"),
        );
    }
}

fn is_html_event_name(event_name: &str, element_name: &str) -> bool {
    match event_name{
            "input_string" => element_name == "input",
            "abort" | // HTMLMediaElement
            "afterprint" | // body
            "animationcancel" | // Element 
            "animationend" | // Element 
            "animationiteration" | // Element 
            "animationstart" | // Element 
            "auxclick" | // Element 
            "beforeinput" | // Element  
            "beforeprint"  | // body
            "beforetoggle" | // HTMLElement, dialog
            "beforeunload" | // body
            "blur" | // body | // Element 
            "cancel" | // input, dialog
            "canplay" | // HTMLMediaElement
            "canplaythrough" | // HTMLMediaElement
            "change" | // input, select, textarea
            "click" | // Element 
            "close" | // dialog
            "compositionend" | // Element 
            "compositionstart" | // Element 
            "compositionupdate" | // Element 
            "contentvisibilityautostatechange" | // Element 
            "contextmenu" | // Element 
            "copy" | // HTMLElement | // Element 
            "cuechange" | // track
            "cut" | // HTMLElement | // Element 
            "dblclick" | // Element 
            "drag" | // HTMLElement
            "dragend" | // HTMLElement
            "dragenter" | // HTMLElement
            "dragleave" | // HTMLElement
            "dragover" | // HTMLElement
            "dragstart" | // HTMLElement
            "drop" | // HTMLElement
            "durationchange" | // HTMLMediaElement
            "emptied" | // HTMLMediaElement
            "ended" | // HTMLMediaElement
            "error" | // body, HTMLElement | // HTMLMediaElement
            "focus" | // body | // Element 
            "focusin" | // Element 
            "focusout" | // Element 
            "formdata" | // form
            "fullscreenchange" | // Element 
            "fullscreenerror" | // Element 
            "gotpointercapture" | // Element 
            "hashchange" | // body
            "input" | // Element 
            "invalid" | // input
            "keydown" | // Element 
            "keyup" | // Element 
            "languagechange" | // body
            "load" | // body, HTMLElement
            "loadeddata" | // HTMLMediaElement
            "loadedmetadata" | // HTMLMediaElement
            "loadstart" | // HTMLMediaElement
            "lostpointercapture" | // Element 
            "message" | // body
            "messageerror" | // body
            "mousedown" | // Element 
            "mouseenter" | // Element 
            "mouseleave" | // Element 
            "mousemove" | // Element 
            "mouseout" | // Element 
            "mouseover" | // Element 
            "mouseup" | // Element 
            "offline" | // body
            "online" | // body
            "pagehide" | // body
            "pagereveal" | // body
            "pageshow" | // body
            "pageswap" | // body
            "paste" | // HTMLElement | // Element 
            "pause" | // HTMLMediaElement
            "play" | // HTMLMediaElement
            "playing" | // HTMLMediaElement
            "pointercancel" | // Element 
            "pointerdown" | // Element 
            "pointerenter" | // Element 
            "pointerleave" | // Element 
            "pointermove" | // Element 
            "pointerout" | // Element 
            "pointerover" | // Element 
            "pointerup" | // Element 
            "popstate" | // body
            "progress" | // HTMLMediaElement
            "ratechange" | // HTMLMediaElement
            "rejectionhandled" | // body
            "reset" | // form
            "resize" | // body
            "scroll" | // Element 
            "scrollend" | // Element 
            "securitypolicyviolation" | // Element 
            "seeked" | // HTMLMediaElement
            "seeking" | // HTMLMediaElement
            "select" | // input
            "selectionchange" | // input
            "slotchange" | // slot
            "stalled" | // HTMLMediaElement
            "storage" | // body
            "submit" | // form
            "suspend" | // HTMLMediaElement
            "timeupdate" | // HTMLMediaElement
            "toggle" | // HTMLElement, dialog 
            "touchcancel" | // Element 
            "touchend" | // Element 
            "touchmove" | // Element 
            "touchstart" | // Element 
            "transitioncancel" | // Element 
            "transitionend" | // Element 
            "transitionrun" | // Element 
            "transitionstart" | // Element 
            "unhandledrejection" | // body
            "unload" | // body
            "volumechange" | // HTMLMediaElement
            "waiting" | // HTMLMediaElement
            "waitingforkey" | // HTMLMediaElement
            "wheel" // Element
                 => true,
                 _ => false,
                 }
}
fn get_static_string(expr_lit: &syn::PatLit) -> Result<String> {
    Ok(match &expr_lit.lit {
        syn::Lit::Str(lit_str) => lit_str.value(),
        syn::Lit::Char(lit_char) => lit_char.value().to_string(),
        syn::Lit::Int(lit_int) => lit_int.base10_digits().to_string(),
        syn::Lit::Float(lit_float) => lit_float.base10_digits().to_string(),
        syn::Lit::Bool(lit_bool) => lit_bool.value.to_string(),
        other_expr => {
            return Err(syn::Error::new(
                other_expr.span(),
                "This type of literal is not suppported",
            ))
        }
    })
}

impl View {
    fn generate_view_state_struct_fields(&self) -> TokenStream {
        let ident = &self.spair_ident;
        let type_name = &self.name;
        quote! {#ident: #type_name,}
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        let ident = &self.spair_ident;
        quote! {#ident,}
    }

    fn generate_code_for_create_view_fn_as_child_node(
        &self,
        parent: &Ident,
        previous: Option<&Ident>,
    ) -> TokenStream {
        let view_state = &self.spair_ident;
        let view_name = &self.name;
        let view_marker = &self.spair_ident_marker;
        let create_view_args = &self.create_view_args;
        let get_marker = match previous {
            Some(previous) => {
                quote! {let #view_marker = #previous.ws_node_ref().next_sibling_ws_node(); }
            }
            None => quote! {let #view_marker= #parent.ws_node_ref().first_ws_node();},
        };
        let create_method_name = Ident::new("create", self.name.span());
        quote! {
            let #view_state = #view_name::#create_method_name(#create_view_args);
            #get_marker
            #parent.insert_new_node_before_a_node(#view_state.root_element(), Some(&#view_marker));
        }
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        self_view_state_ident: &Ident,
    ) -> TokenStream {
        let view_state = &self.spair_ident;
        if self.update_view_method_name.is_some() {
            let update_view_args = &self.update_view_args;
            let update_method_name = &self.update_view_method_name;
            quote! {
                #self_view_state_ident.#view_state.#update_method_name(#update_view_args);
            }
        } else {
            quote! {}
        }
    }
}
