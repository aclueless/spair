use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Expr, ExprCall, Ident, Result};

use crate::{view, ItemCounter, MultiErrors};

pub(crate) enum Element {
    Text(Text),
    HtmlElement(HtmlElement),
    View(View),
    // Component(Component),
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

    meta: HtmlElementMeta,
}

pub(crate) struct View {
    pub(crate) name: Ident,
    create_view: Call,
    update_view: Call,
    spair_ident: Ident,
    spair_ident_marker: Ident,
}

struct Call {
    name: Ident,
    args: Punctuated<Expr, Comma>,
}

// pub struct Component {
//     name: Ident,
//     props: Punctuated<Expr, Token![,]>,
// }

struct Attribute {
    stage: Stage,
    name_string: String,
    name_ident: Ident,
    value: Expr,

    spair_store_index: usize,
    is_event_listener: bool,
}

enum Stage {
    HtmlString(String),
    Creation,
    Update,
}

impl Element {
    // Collecting element stops on the first error. It is a bit difficult to collect all the errors at the same time.
    pub fn with_expr(expr: Expr) -> Result<Self> {
        const EXPECTED_HTML_CONSTRUCTION_EXPR: &str = r#"Expected an HTML construction expression that looks like:
div(
    id = "some_id",
    class = "some_class_name",
    button(
        click = some_handler,
        text("some text", some_text),
    ),
    div(
       text("some text"),
    )
)"#;
        let expr = match expr {
            Expr::Call(expr_call) => expr_call,
            other_expr => {
                return Err(syn::Error::new(
                    other_expr.span(),
                    EXPECTED_HTML_CONSTRUCTION_EXPR,
                ));
            }
        };
        let mut item_counter = ItemCounter::new();
        let element = Self::with_expr_call(expr, &mut item_counter)?
            .pop()
            .unwrap();
        Ok(element)
    }

    fn with_expr_call(expr: ExprCall, item_counter: &mut ItemCounter) -> Result<Vec<Self>> {
        // div(a=b, c=d, span(...), ...), or
        // view::ViewName(...), or
        // comp::ComponentName(...)
        let element = match *expr.func {
            Expr::Path(mut expr_path) if expr_path.path.segments.len() == 1 => {
                // Don't expect PathSegment.arguments, just ignore it now, should report an error?
                let html_tag = expr_path.path.segments.pop().unwrap().into_value().ident;
                if html_tag == "text" {
                    if expr.args.is_empty() {
                        return Err(syn::Error::new(expr.paren_token.span.span(), "Empty text?"));
                    }
                    let mut text_nodes = Vec::new();
                    let text_node_count = expr.args.len();
                    for (index, expr) in expr.args.into_iter().enumerate() {
                        let next_node_is_text = index + 1 < text_node_count;
                        let text_node = match expr {
                            Expr::Lit(expr_lit) => {
                                let text_value = get_static_string(&expr_lit)?;
                                Text {
                                    shared_name: html_tag.clone(),
                                    stage: Stage::HtmlString(text_value),
                                    value: Expr::Lit(expr_lit),
                                    spair_ident: item_counter.new_ident_text(),
                                    next_node_is_a_text: next_node_is_text,
                                }
                            }
                            other_expr => Text {
                                shared_name: html_tag.clone(),
                                stage: Stage::Update,
                                value: other_expr,
                                spair_ident: item_counter.new_ident_text(),
                                next_node_is_a_text: next_node_is_text,
                            },
                        };
                        text_nodes.push(text_node);
                    }
                    return Ok(text_nodes.into_iter().map(Element::Text).collect());
                }
                let html = HtmlElement::with_name_n_args(html_tag, expr.args, item_counter)?;
                Element::HtmlElement(html)
            }
            Expr::Path(mut expr_path) if expr_path.path.segments.len() == 2 => {
                // Don't expect PathSegment.arguments, just ignore it now, should report an error?
                let name = expr_path.path.segments.pop().unwrap().into_value().ident;
                let type_ident = expr_path.path.segments.pop().unwrap().into_value().ident;
                if type_ident == "view" {
                    let view = View::collect(name, expr.args, item_counter)?;
                    Element::View(view)
                } else if type_ident != "comp" {
                    // let comp = Component::collect(name, expr.args)?;
                    // Element::Component(comp)
                    todo!()
                } else {
                    return Err(syn::Error::new(
                        type_ident.span(),
                        "Expected 'view' or 'comp'",
                    ));
                }
            }
            other_expr => {
                return Err(syn::Error::new(
                    other_expr.span(),
                    "Expected HTML tags (div, input...), ViewName, or ComponentName",
                ));
            }
        };
        Ok(vec![element])
    }

    fn span(&self) -> Span {
        match self {
            Element::HtmlElement(html_element) => html_element.name.span(),
            Element::Text(text) => text.shared_name.span(),
            Element::View(view) => view.name.span(),
            // Element::Component(component) => component.name.span(),
        }
    }

    pub fn check_html_multi_errors(&self, errors: &mut MultiErrors) {
        match self {
            Element::Text(_text) => {}
            Element::HtmlElement(html_element) => html_element.check_html_multi_errors(errors),
            Element::View(_view) => {}
        }
    }

    fn construct_html(&self, html_string: &mut String) {
        match self {
            Element::Text(text) => {
                if let Stage::HtmlString(text_value) = &text.stage {
                    html_string.push_str(&text_value);
                } else {
                    html_string.push('?');
                }
            }
            Element::HtmlElement(html_element) => html_element.construct_html(html_string),
            Element::View(_) => {
                html_string.push_str("<!--view-->");
            }
        }
    }

    pub(crate) fn prepare_items_for_generating_code(&mut self, update_stage_variables: &[String]) {
        match self {
            Element::Text(text) => text.prepare_items_for_generating_code(update_stage_variables),
            Element::HtmlElement(html_element) => {
                html_element.prepare_items_for_generating_code(update_stage_variables)
            }
            Element::View(_) => {}
        }
    }

    pub(crate) fn generate_view_state_struct_fields(&self) -> TokenStream {
        match self {
            Element::Text(text) => text.generate_view_state_struct_fields(),
            Element::HtmlElement(html_element) => html_element.generate_view_state_struct_fields(),
            Element::View(view) => view.generate_view_state_struct_fields(),
        }
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        match self {
            Element::Text(text) => text.generate_fields_for_view_state_instance(),
            Element::HtmlElement(html_element) => {
                html_element.generate_fields_for_view_state_instance()
            }
            Element::View(view) => view.generate_fields_for_view_state_instance(),
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
            Element::HtmlElement(html_element) => {
                html_element.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
            Element::View(view) => {
                view.generate_code_for_create_view_fn_as_child_node(parent, previous)
            }
        }
    }

    fn spair_ident_to_get_next_node(&self) -> &Ident {
        match self {
            Element::Text(text) => &text.spair_ident,
            Element::HtmlElement(html_element) => &html_element.meta.spair_ident,
            Element::View(view) => &view.spair_ident_marker,
        }
    }

    fn generate_code_for_update_view_fn(&self, view_state_ident: &Ident) -> TokenStream {
        match self {
            Element::Text(text) => {
                text.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::HtmlElement(html_element) => {
                html_element.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
            Element::View(view) => {
                view.generate_code_for_update_view_fn_as_child_node(view_state_ident)
            }
        }
    }
}

impl Text {
    fn prepare_items_for_generating_code(&mut self, update_stage_variables: &[String]) {
        dbg!("prepare_items_for_generating_code");
        if let Stage::HtmlString(_) = &self.stage {
            return;
        };
        if view::expr_has_ref_to(&self.value, update_stage_variables) {
            self.stage = Stage::Update;
            dbg!("... in text: it's an update");
        } else {
            self.stage = Stage::Creation;
            dbg!("... in text: it's a create");
        }
    }

    fn generate_view_state_struct_fields(&self) -> TokenStream {
        if matches!(self.stage, Stage::Update) {
            let ident = &self.spair_ident;
            quote! {#ident: Text,}
        } else {
            quote! {}
        }
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
            Stage::HtmlString(s) => s.chars().count(),
            Stage::Creation => 1,
            Stage::Update => 1,
        };
        let text = &self.value;
        let get_text_node = |first_text_method_name, next_text_method_name| {
            let first_text_method_name = Ident::new(first_text_method_name, Span::call_site());
            let next_text_method_name = Ident::new(next_text_method_name, Span::call_site());
            match previous {
                None => {
                    quote! { let #text_node = #parent.ws_node_ref().#first_text_method_name(); }
                }
                Some(previous) => {
                    if self.next_node_is_a_text {
                        quote! {
                            let #text_node = #previous.ws_node_ref().#next_text_method_name();
                            #text_node.split_text(#offset);
                        }
                    } else {
                        quote! { let #text_node = #previous.ws_node_ref().#next_text_method_name(); }
                    }
                }
            }
        };
        match self.stage {
            Stage::Creation => {
                let get_text_node = get_text_node("first_ws_text", "next_sibling_ws_text");
                quote! {
                    #get_text_node
                    #text_node.set_text_content(#text);
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
                    quote!{#get_text_node}
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
    ) -> Result<HtmlElement> {
        let spair_ident = item_counter.new_ident_element();
        let mut attributes = Vec::new();
        let mut children: Vec<Element> = Vec::new();
        for expr in args.into_iter() {
            match expr {
                Expr::Assign(expr_assign) => {
                    if let Some(element) = children.last() {
                        return Err(syn::Error::new(
                            element.span(),
                            "An attribute can not appear after a text or child node",
                        ));
                    }
                    attributes.push(Attribute::with_expr_assign(expr_assign)?)
                }
                Expr::Call(expr_call) => {

                    let vec = Element::with_expr_call(expr_call, item_counter)?;
                    if let Some(Element::Text(_)) = vec.first() {
                        if let Some(Element::Text(last))=children.last_mut() {
                            last.next_node_is_a_text = true;
                        }
                    }
                    children.extend(vec)
                },
                other_expr => return Err(syn::Error::new(other_expr.span(), "Expected an attribute assignment `class = some_value` or child element as `text(some_value)`, `div(...)`, `view::ViewName(...)`, `comp::ComponentName(...)`")),
            }
        }
        Ok(HtmlElement {
            name,
            attributes,
            children,
            root_element: false,
            meta: HtmlElementMeta {
                spair_element_capacity: 0,
                spair_ident,
            },
        })
    }

    fn count_spair_element_capacity(&mut self) {
        let mut number_of_attribute_need_to_store = 0;
        for attribute in self.attributes.iter_mut().filter(|attribute| {
            attribute.is_event_listener || matches!(&attribute.stage, Stage::Update)
        }) {
            attribute.spair_store_index = number_of_attribute_need_to_store;
            number_of_attribute_need_to_store += 1;
        }

        self.meta.spair_element_capacity = number_of_attribute_need_to_store;
    }

    pub fn check_html(&self) -> Result<()> {
        let mut errors = MultiErrors::default();
        self.check_html_multi_errors(&mut errors);
        errors.report_error()
    }

    fn check_html_multi_errors(&self, errors: &mut MultiErrors) {
        self.check_html_tag(errors);
        for attribute in self.attributes.iter() {
            attribute.check_html(errors);
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
        self.construct_html(&mut html_string);
        html_string
    }

    fn construct_html(&self, html_string: &mut String) {
        let html_tag = self.name.to_string();
        let (open_closing, close_1, close_2, close_3) = match html_tag.as_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "source" | "track" | "wbr" => (" />", "", "", ""),
            html_tag => (">", "</", html_tag, ">"),
        };
        html_string.push('<');
        html_string.push_str(&html_tag);
        self.construct_html_attributes(html_string);
        html_string.push_str(&open_closing);
        self.construct_html_children(html_string);
        html_string.push_str(&close_1);
        html_string.push_str(&close_2);
        html_string.push_str(&close_3);
    }

    fn construct_html_attributes(&self, html_string: &mut String) {
        for attribute in self.attributes.iter() {
            attribute.construct_html(html_string);
        }
    }

    fn construct_html_children(&self, html_string: &mut String) {
        for element in self.children.iter() {
            element.construct_html(html_string);
        }
    }

    pub(crate) fn prepare_items_for_generating_code(&mut self, update_stage_variables: &[String]) {
        let mut spair_store_index = 0;
        for attribute in self.attributes.iter_mut() {
            if let Stage::HtmlString(_) = &attribute.stage {
                continue;
            }
            if view::expr_has_ref_to(&attribute.value, update_stage_variables) {
                attribute.stage = Stage::Update;
                attribute.spair_store_index = spair_store_index;
                spair_store_index += 1;
            } else {
                attribute.stage = Stage::Creation;
            }
        }
        for element in self.children.iter_mut() {
            element.prepare_items_for_generating_code(update_stage_variables);
        }
        self.count_spair_element_capacity();
    }

    pub fn generate_view_state_struct_fields(&self) -> TokenStream {
        let ident = &self.meta.spair_ident;
        let self_element = if self.root_element || self.meta.spair_element_capacity > 0 {
            quote! {#ident: Element, }
        } else {
            quote! {}
        };
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
    fn generate_view_state_instance(&self, view_state_struct_name: &Ident) -> TokenStream {
        let ident = &self.meta.spair_ident;
        let self_element = if self.root_element || self.meta.spair_element_capacity > 0 {
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
            #view_state_struct_name{
                #self_element
                #view_state_fields
            }
        }
    }

    fn generate_fields_for_view_state_instance(&self) -> TokenStream {
        self.children
            .iter()
            .map(|v| v.generate_fields_for_view_state_instance())
            .collect()
    }

    pub(crate) fn root_element_type(&self) -> Ident {
        Ident::new("Element", Span::call_site())
    }

    pub(crate) fn root_element_ident(&self) -> &Ident {
        &self.meta.spair_ident
    }

    pub fn generate_code_for_create_view_fn_of_a_view(
        &self,
        view_state_struct_name: &Ident,
        html_string: &str,
    ) -> TokenStream {
        let first_part = self.generate_code_for_create_view_fn(html_string);
        let construct_view_state_instance =
            self.generate_view_state_instance(view_state_struct_name);
        quote! {
            #first_part
            #construct_view_state_instance
        }
    }

    pub fn generate_code_for_create_view_fn_of_a_component(
        &self,
        view_state_struct_name: &Ident,
        html_string: &str,
    ) -> TokenStream {
        let root_element = &self.meta.spair_ident;
        let first_part = self.generate_code_for_create_view_fn(html_string);
        let construct_view_state_instance =
            self.generate_view_state_instance(view_state_struct_name);
        quote! {
            #first_part
            (#root_element.ws_element().clone(), #construct_view_state_instance)
        }
    }

    fn generate_code_for_create_view_fn(&self, html_string: &str) -> TokenStream {
        let root_element = &self.meta.spair_ident;
        let capacity = self.meta.spair_element_capacity;
        let attribute_setting = self.generate_attribute_code_for_create_view_fn();
        let children = self.generate_children_code_for_create_view_fn();
        quote! {
            const HTML_STRING: &str = #html_string;
            let mut #root_element = Element::with_html(HTML_STRING, #capacity);
            #attribute_setting
            #children
        }
    }

    fn generate_attribute_code_for_create_view_fn(&self) -> TokenStream {
        let element = &self.meta.spair_ident;

        self.attributes
            .iter()
            .map(|v| v.generate_attribute_setting(element))
            .collect()
    }

    fn generate_children_code_for_create_view_fn(&self) -> TokenStream {
        let element = &self.meta.spair_ident;
        let mut previous = None;
        self.children
            .iter()
            .map(|v| {
                let code = v.generate_code_for_create_view_fn(&element, previous);
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
                quote! {let #element = #previous.ws_node_ref().next_sibling_element();}
            }
            None => quote! { let #element = #parent.ws_node_ref().first_element(); },
        };
        let get_element = if self.meta.spair_element_capacity > 0 {
            let capacity = self.meta.spair_element_capacity;
            quote! {let #element = #element.create_element_with_capacity(#capacity);}
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

        self.attributes
            .iter()
            .map(|v| v.generate_attribute_update_setting(element, view_state_ident))
            .collect()
    }

    fn generate_children_code_for_update_view_fn(&self, view_state_ident: &Ident) -> TokenStream {
        self.children
            .iter()
            .map(|v| v.generate_code_for_update_view_fn(view_state_ident))
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
}

impl Attribute {
    fn with_expr_assign(expr: syn::ExprAssign) -> Result<Self> {
        let name_ident = match *expr.left {
            Expr::Path(mut expr_path) if expr_path.path.segments.len() == 1 => {
                expr_path.path.segments.pop().unwrap().into_value().ident
            }
            other_expr => {
                let name = match &other_expr {
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
                    _ => "other",
                };
                return Err(syn::Error::new(
                    other_expr.span(),
                    &format!("Expected a single identifier as an HTML attribute name, found expression type: {name}"),
                ));
            }
        };
        let stage = match &*expr.right {
            Expr::Lit(expr_lit) => {
                let s = get_static_string(expr_lit)?;
                Stage::HtmlString(s)
            }
            _ => Stage::Update,
        };
        let name_string = name_ident.to_string();
        let is_event_listener = is_html_event_name(&name_string);
        let attribute = Attribute {
            stage,
            name_string,
            name_ident,
            value: *expr.right,

            is_event_listener,
            spair_store_index: 0,
        };
        Ok(attribute)
    }

    fn check_html(&self, errors: &mut MultiErrors) {
        if self.is_event_listener || is_html_attribute_name(&self.name_string) {
            return;
        }
        if self.name_string.starts_with("data") {
            return;
        }
        errors.add(self.name_ident.span(), "unknown attribute");
    }

    fn construct_html(&self, html_string: &mut String) {
        if let Stage::HtmlString(value) = &self.stage {
            html_string.push(' ');
            html_string.push_str(&self.name_string);
            html_string.push_str("='");
            html_string.push_str(value);
            html_string.push_str("'");
        }
    }

    fn generate_attribute_setting(&self, element: &Ident) -> TokenStream {
        let name_ident = &self.name_ident;
        let index = self.spair_store_index;
        let expr = match &self.value {
            Expr::Closure(expr_closure) => quote! {context.comp.callback_arg_mut(#expr_closure)},
            event_listener => quote! {#event_listener},
        };
        if self.is_event_listener {
            return quote! {#element.#name_ident(#index, #expr);};
        }
        match self.name_string.as_str() {
            "replace_at_element_id" => {
                quote! {#element.#name_ident(#expr);}
            }
            "class_if" => quote! {},
            _other_name => quote! {},
        }
    }

    fn generate_attribute_update_setting(
        &self,
        element: &Ident,
        view_state_ident: &Ident,
    ) -> TokenStream {
        if let Stage::Update = &self.stage {
            quote! { #view_state_ident.#element. update a attribute need implementation }
        } else {
            quote! {}
        }
    }
}

fn is_html_attribute_name(name: &str) -> bool {
    match name {
            "replace_at_element_id" | // spair attribute: there is an element (element A) given in html document (which has `id` given by this attribute). Spair will put this element (created in spair component) in place of the element A. 
            "accept" | // form, input
            "accept_charset" | // form
            "accesskey" | // global attribute
            "action" | // form
            "allow" | // iframe
            "alt" | // area, img, input
            "as" | // link
            "async" | // script
            "autocapitalize" | // global
            "autocomplete" | // form, input, select, textarea
            "autoplay" | // audio, video
            "capture" | // input
            "charset" | // meta
            "checked" | // input
            "cite" | // blockquote, del, ins, q
            "class" | // global
            "cols" | // textarea
            "colspan" | // td, th
            "content" | // meta
            "contenteditable" | // global
            "controls" | // audio, video
            "coords" | // area
            "crossorigin" | // audio, img, link, script, video
            "csp" | // iframe
            "data" | // object
//            "data-" | // global
            "datetime" | // del, ins, time
            "decoding" | // img
            "default" | // track
            "defer" | // script
            "dir" | // global
            "dirname" | // input, textarea
            "disabled" | // button, fieldset, input, optgroup, option, select, textarea
            "download" | // a, area
            "draggable" | // global
            "enctype" | // form
            "enterkeyhint" | // textarea, contenteditable
            "for" | // label, output
            "form" | // button, fieldset, input, label, meter, object, output, progress, select, textarea
            "formaction" | // input, button
            "formenctype" | // input, button
            "formmethod" | // input, button
            "formnovalidate" | // input, button
            "formtarget" | // input, button
            "headers" | // td, th
            "height" | // canvas, embed, iframe, img, input, object, video
            "hidden" | // golbal
            "high" | // meter
            "href" | // a, area, base, link
            "hreflang" | // a, link
            "http-equiv" | // meta
            "id" | // global
            "integrity" | // link, script
            "inputmode" | // textarea, contenteditable
            "ismap" | // img
            "itemprop" | // global
            "kind" | // track
            "label" | // optgroup, option, track
            "lang" | // global
            "loading" | // img, iframe
            "list" | // input
            "loop" | // audio, video
            "low" | // meter
            "max" | // input, meter, progress
            "maxlength" | // input, textarea
            "minlength" | // input, textarea
            "media" | // a, area, link, source, style
            "method" | // form
            "min" | // input, meter
            "multiple" | // input, select
            "muted" | // audio, video
            "name" | // button, form, fieldset, iframe, input, object, output, select, textarea, map, meta, param
            "novalidate" | // form
            "open" | // details, dialog
            "optimum" | // meter
            "pattern" | // output
            "ping" | // a, area
            "placeholder" | // input, textarea
            "playsinline" | // video
            "poster" | // video
            "preload" | // audio, video
            "readonly" | // input, textarea
            "referrerpolicy" | // a, area, iframe, img, link, script
            "rel" | // a, area, link
            "required" | // input, select, textarea
            "reversed" | // ol
            "role" | // global
            "rows" | // textarea
            "rowspan" | // td, th
            "sandbox" | // iframe
            "scope" | // th
            "selected" | // option
            "shape" | // a, area
            "size" | // input, select
            "sizes" | // link, img, source
            "slot" | // global
            "span" | // col, colgroup
            "spellcheck" | // global
            "src" | // audio, embed, iframe, img, input, script, source, track, video
            "srcdoc" | // iframe
            "srclang" | // track
            "srcset" | // img, source
            "start" | // ol
            "step" | // input
            "style" | // global
            "tableindex" | // global
            "target" | // a, area, base, form
            "title" | // global
            "translate" | // global
            "type" | // button, input, embed, object, ol, script, source, style, menu, link
            "usemap" | // img, input, object
            "value" | // button, data, input, li, meter, option, progress, param
            "width" | // canvas, embed, iframe, img, input, object, video
            "wrap"  // textarea
            => true,
            _ => false,
            }
}

fn is_html_event_name(name: &str) -> bool {
    match name{
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
    fn collect(
        name: Ident,
        args: Punctuated<Expr, syn::token::Comma>,
        item_counter: &mut ItemCounter,
    ) -> Result<Self> {
        let mut args = args.into_iter();
        let create_view = collect_call(
            &name,
            "create_view",
            &mut args,
            "Expected `create_view` to be given like `ViewName(create_view(arg1, arg2, ...))`",
        )?;
        let update_view = collect_call(
            &name,
            "update_view",
            &mut args,
            "Expected `update_view` (after `create_view`) to be given like `ViewName(create_view(...), update_view(arg1, arg2, ...))`",
        )?;
        Ok(View {
            name,
            create_view,
            update_view,
            spair_ident: item_counter.new_ident_view(),
            spair_ident_marker: item_counter.new_ident("_view_marker"),
        })
    }

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
        let Call {
            name: create_view,
            args: create_view_args,
        } = &self.create_view;
        let get_marker = match previous {
            Some(previous) => {
                quote! {let #view_marker = #previous.ws_node_ref().next_sibling_ws_node(); }
            }
            None => quote! {let #view_marker= #parent.ws_node_ref().first_ws_node();},
        };
        quote! {
            let #view_state = #view_name::#create_view(#create_view_args);
            #get_marker
            #parent.insert_new_node_before_a_node(#view_state.root_element(), Some(&#view_marker));
        }
    }

    fn generate_code_for_update_view_fn_as_child_node(
        &self,
        self_view_state_ident: &Ident,
    ) -> TokenStream {
        let view_state = &self.spair_ident;
        let Call { name, args } = &self.update_view;
        quote! {
            #self_view_state_ident.#view_state.#name(#args);
        }
    }
}

fn collect_call(
    view_name: &Ident,
    func_name: &str,
    args: &mut syn::punctuated::IntoIter<Expr>,
    error_message: &str,
) -> std::result::Result<Call, syn::Error> {
    let Some(create_view) = args.next() else {
        return Err(syn::Error::new(view_name.span(), error_message));
    };
    let create_view = match create_view {
        Expr::Call(expr) => expr,
        other_expr => return Err(syn::Error::new(other_expr.span(), error_message)),
    };
    let ident = match *create_view.func {
        Expr::Path(mut expr_path) if expr_path.path.is_ident(func_name) => {
            expr_path.path.segments.pop().unwrap().into_value().ident
        }
        other_expr => return Err(syn::Error::new(other_expr.span(), error_message)),
    };
    Ok(Call {
        name: ident,
        args: create_view.args,
    })
}
