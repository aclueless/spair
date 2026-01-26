use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Ident, punctuated::Punctuated, spanned::Spanned, token::Comma};

use super::{
    Item, ItemCounter, Items, LastNode, MultiErrors, SubMod, expr_as_ident, get_static_string,
    stage::{Stage, StagePicker},
};

const REPLACE_AT_ELEMENT_ID: &str = "replace_at_element_id";
const HREF_WITH_ROUTING: &str = "href_with_routing";
const HREF_STR: &str = "href_str";
const INNER_HTML: &str = "unsafely_set_inner_html";

const SET_NODE_REF_TO: &str = "set_node_ref_to";

#[derive(Debug)]
pub struct Element {
    html_tag_ident: Ident,
    attributes: Vec<Attribute>,
    events: Vec<Attribute>,
    select_value_setting: Option<Attribute>,
    children: Items,

    at_root: bool,
    need_this_as_a_parent_for_updating_children: bool,
    spair_element_capacity: usize,
    spair_ident: Ident,
}

struct Attribute {
    stage: Stage,
    attribute_name_in_rust: Ident,
    attribute_name_in_string: String,
    attribute_value: Expr,

    is_event_attribute: bool,
    spair_store_index: usize,
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attribute_name_in_string)
    }
}

impl Attribute {
    fn check_html(&self, element_name: &str, errors: &mut MultiErrors) {
        if self.is_event_attribute {
            return;
        }
        if self.attribute_name_in_string.starts_with("data-") {
            return;
        }
        if self.attribute_name_in_string.starts_with("aria-") {
            return;
        }
        match self.attribute_name_in_string.as_str() {
            "class_if" => {
                let message = "`class_if` requires a tuple of 2 expressions as `(boolean_expr, some_class_name)`";
                match &self.attribute_value {
                    Expr::Tuple(expr) => {
                        if expr.elems.len() < 2 {
                            errors.error_at(expr.span(), message);
                        }
                        if let Some(third) = expr.elems.get(2) {
                            errors.error_at(
                                third.span(),
                                "`class_if` requires exactly 2 expressions",
                            );
                        }
                    }
                    other => errors.error_at(other.span(), message),
                }
            }
            "class_or" => {
                let message = "`class_or` requires a tuple of 3 expressions as `(boolean_expr, some_class_name, other_class_name)`";
                match &self.attribute_value {
                    Expr::Tuple(expr) => {
                        if expr.elems.len() < 3 {
                            errors.error_at(expr.span(), message);
                        }
                        if let Some(forth) = expr.elems.get(3) {
                            errors.error_at(
                                forth.span(),
                                "`class_or` requires exactly 3 expressions",
                            );
                        }
                    }
                    other => errors.error_at(other.span(), message),
                }
            }
            REPLACE_AT_ELEMENT_ID => {
                if self.stage == Stage::Update {
                    errors.error_at(
                        self.attribute_value.span(),
                        &format!("Value for {REPLACE_AT_ELEMENT_ID} can not get value from update fn, it must be a literal or a variable in create fn"),
                    );
                }
            }
            _ => {
                check_html_attribute_name(
                    &self.attribute_name_in_rust,
                    &self.attribute_name_in_string,
                    element_name,
                    errors,
                );
            }
        }
    }

    fn generate_html_string(&self, html_string: &mut String) {
        match self.attribute_name_in_string.as_str() {
            REPLACE_AT_ELEMENT_ID | HREF_WITH_ROUTING => {}
            INNER_HTML => {
                // Not expect an inner HTML at static stage
            }
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

    fn generate_fn_create_element_creation_code(
        &self,
        _element_name: &str,
        element: &Ident,
    ) -> TokenStream {
        let must_handle_in_create_fn = self.stage == Stage::Creation
            || match self.attribute_name_in_string.as_str() {
                REPLACE_AT_ELEMENT_ID | HREF_WITH_ROUTING => true,
                _ => false,
            };
        if must_handle_in_create_fn.not() {
            return quote! {};
        }
        if self.is_event_attribute {
            return self.generate_code_for_adding_event_handlers(&quote! {#element});
        }
        let attribute_value = &self.attribute_value;
        match self.attribute_name_in_string.as_str() {
            REPLACE_AT_ELEMENT_ID => {
                quote! {#element.replace_at_element_id(#attribute_value);}
            }
            HREF_WITH_ROUTING => {
                if self.stage == Stage::Creation {
                    quote! {
                        #element.add_click_event_to_handle_routing();
                        #element.href_with_routing(#attribute_value);
                    }
                } else {
                    quote! {
                        #element.add_click_event_to_handle_routing();
                    }
                }
            }
            HREF_STR => quote! {#element.set_str_attribute("href",#attribute_value);},
            INNER_HTML => quote! {#element.unsafely_set_inner_html(#attribute_value);},
            "id" => quote! {#element.set_id(#attribute_value);},
            "class" => quote! {#element.add_class(#attribute_value);},
            "class_if" => {
                if let Expr::Tuple(expr) = attribute_value {
                    let condition_expr = &expr.elems[0];
                    let class_name = &expr.elems[1];
                    quote! {
                        #element.class_if(#condition_expr,#class_name);
                    }
                } else {
                    quote! {}
                }
            }
            "disabled" => quote! {#element.set_bool_attribute("disabled", #attribute_value);},
            "enabled" => quote! {#element.set_bool_attribute("disabled", !(#attribute_value));},
            other_attribute_name => {
                let message = format!(
                    "`{other_attribute_name}` attribute is not implemented for creation stage yet.",
                );
                quote! {compile_error!(#message);}
            }
        }
    }

    fn generate_code_for_adding_event_handlers(&self, element: &TokenStream) -> TokenStream {
        let index = self.spair_store_index;
        let attribute_name = &self.attribute_name_in_rust;
        let attribute_value = &self.attribute_value;
        quote! {#element.#attribute_name(#index, #attribute_value);}
    }

    fn generate_view_state_fn_update_code(
        &self,
        element_name: &str,
        element: &TokenStream,
    ) -> TokenStream {
        if self.stage != Stage::Update {
            return quote! {};
        }
        let attribute_value = &self.attribute_value;
        let index = self.spair_store_index;
        if self.is_event_attribute {
            return self.generate_code_for_adding_event_handlers(element);
        }
        match self.attribute_name_in_string.as_str() {
            REPLACE_AT_ELEMENT_ID => quote! {},
            HREF_STR => {
                quote! {#element.set_str_attribute_at_index(#index, "href", #attribute_value);}
            }
            HREF_WITH_ROUTING => {
                quote! {#element.href_with_routing_at_index(#index,#attribute_value);}
            }
            INNER_HTML => {
                return quote! {#element.set_inner_html_is_not_safe_at_index(#index,#attribute_value);};
            }
            "class" => quote! {
                #element.update_class(#index, #attribute_value);
            },
            "class_if" => {
                if let Expr::Tuple(expr) = attribute_value {
                    let condition_expr = &expr.elems[0];
                    let class_name = &expr.elems[1];
                    quote! {
                        #element.class_if_at_index(#index, #condition_expr, #class_name);
                    }
                } else {
                    quote! {}
                }
            }
            "class_or" => {
                if let Expr::Tuple(expr) = attribute_value {
                    let condition_expr = &expr.elems[0];
                    let first_class_name = &expr.elems[1];
                    let second_class_name = &expr.elems[2];
                    quote! {
                        #element.class_or_at_index(#index, #condition_expr, #first_class_name, #second_class_name);
                    }
                } else {
                    quote! {}
                }
            }
            "disabled" => {
                quote! {#element.set_bool_attribute_at_index(#index, "disabled", #attribute_value);}
            }
            "enabled" => {
                quote! {#element.set_bool_attribute_at_index(#index, "disabled", !(#attribute_value));}
            }
            "value" => match element_name {
                "input" => {
                    quote! {#element.set_input_value_at_index(#index, #attribute_value);}
                }
                "textarea" => {
                    quote! {#element.set_textarea_value_at_index(#index, #attribute_value);}
                }
                "select" => {
                    quote! {#element.set_select_value_at_index(#index, #attribute_value);}
                }
                "option" => {
                    quote! {#element.set_option_value_at_index(#index, #attribute_value);}
                }
                _ => {
                    let message = format!(
                        "`value` is not an attribute or property of `{element_name}`. Actually, Spair's proc-macros must be update to the report error earlier and at the exact location of the code."
                    );
                    quote! {comple_error(#message);}
                }
            },
            "checked" => {
                quote! {#element.set_input_checked_at_index(#index, #attribute_value);}
            }
            other_attribute_name => {
                let message = format!(
                    "`{other_attribute_name}` attribute is not implemented for update stage yet.",
                );
                quote! {compile_error!(#message);}
            }
        }
    }
}

fn check_html_attribute_name(
    attribute_ident: &Ident,
    attribute_name: &str,
    element_name: &str,
    errors: &mut MultiErrors,
) {
    let elements: &[&str] = match attribute_name {
        // spair attribute: there must be an element (element A) given in the html document
        // (usually named `index.html`). The element A must have an id which is the value of
        // this attribute). Spair will replace this element (created in spair component) in
        // the place of the element A.
        REPLACE_AT_ELEMENT_ID => return,
        SET_NODE_REF_TO => return,
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
            errors.error_at(
                attribute_ident.span(),
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
        "innerHTML" => {
            errors.error_at(
                attribute_ident.span(),
                &format!("Explicitly use `{INNER_HTML}` instead of just `innerHTML`"),
            );
            return;
        }
        INNER_HTML => return,
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
        errors.error_at(
            attribute_ident.span(),
            &format!("Unknown attribute for `<{element_name}>`"),
        );
    }
}

impl Element {
    pub fn first_span(&self) -> Span {
        self.html_tag_ident.span()
    }

    pub fn spair_ident(&self) -> &Ident {
        &self.spair_ident
    }

    pub fn new(
        at_root: bool,
        html_tag_ident: Ident,
        args: Punctuated<Expr, Comma>,
        stage_picker: &StagePicker,
        item_counter: &mut ItemCounter,
        errors: &mut MultiErrors,
    ) -> Element {
        let html_tag_in_string = html_tag_ident.to_string();
        let spair_ident = item_counter.new_ident_element(&html_tag_in_string);
        let mut element = Self {
            html_tag_ident,
            attributes: Vec::new(),
            events: Vec::new(),
            select_value_setting: None,
            children: Items::default(),
            at_root,
            need_this_as_a_parent_for_updating_children: false,
            spair_element_capacity: 0,
            spair_ident,
        };

        for expr in args.into_iter() {
            element.collect_from_expr(
                false, // not a root element anymore
                &html_tag_in_string,
                expr,
                item_counter,
                stage_picker,
                errors,
            );
        }

        element
    }

    pub fn collect_from_expr(
        &mut self,
        at_root: bool,
        html_tag_in_string: &str,
        expr: syn::Expr,
        item_counter: &mut ItemCounter,
        stage_picker: &StagePicker,
        errors: &mut MultiErrors,
    ) {
        match expr {
            Expr::Assign(expr_assign) => match self.children.items.last() {
                Some(last_element) => {
                    errors.error_at(
                        last_element.first_span(),
                        "An attribute can not appear after a text or child node",
                    );
                }
                None => {
                    self.collect_attribute(&html_tag_in_string, expr_assign, stage_picker, errors);
                }
            },
            other_expr => {
                self.children.collect_from_expr(
                    at_root,
                    other_expr,
                    stage_picker,
                    item_counter,
                    errors,
                );
            }
        }
    }

    pub fn collect_attribute(
        &mut self,
        html_tag_in_string: &str,
        expr_assign: syn::ExprAssign,
        stage_picker: &StagePicker,
        errors: &mut super::MultiErrors,
    ) {
        let mut attribute_name_in_rust = match expr_as_ident(
            *expr_assign.left,
            "Expected a single identifier as an HTML attribute name",
        ) {
            Ok(attribute_name_in_rust) => attribute_name_in_rust,
            Err(error) => {
                errors.combine(error);
                return;
            }
        };
        let attribute_str_in_spair_macro = attribute_name_in_rust.to_string();
        let mut is_event_attribute = false;
        let attribute_name_in_string =
            if let Some(html_event_name) = attribute_str_in_spair_macro.strip_prefix("on_") {
                is_event_attribute = is_html_event_name(html_event_name, html_tag_in_string);
                if !is_event_attribute {
                    errors.error_at(
                        attribute_name_in_rust.span(),
                        &format!("Unknown event `{html_event_name}`"),
                    );
                    return;
                }
                attribute_name_in_rust = Ident::new(html_event_name, attribute_name_in_rust.span());
                html_event_name.to_string()
            } else if let Some(key) = attribute_str_in_spair_macro.strip_prefix("r#") {
                key.to_string()
            } else if attribute_str_in_spair_macro.starts_with("data_") {
                attribute_str_in_spair_macro.replace("_", "-")
            } else if attribute_str_in_spair_macro.starts_with("aria_") {
                attribute_str_in_spair_macro.replacen("aria_", "aria-", 1)
            } else {
                attribute_str_in_spair_macro
            };
        let stage = match &*expr_assign.right {
            Expr::Lit(expr_lit) => {
                if let Some(text_value) = get_static_string(&expr_lit, errors) {
                    if is_event_attribute {
                        errors.error_at(
                            expr_lit.span(),
                            "An event cannot have a literal value. An event value must be a callback.",
                        );
                    }
                    Stage::HtmlString(text_value)
                } else {
                    return;
                }
            }
            other_expr => stage_picker.stage_of(other_expr),
        };
        let attribute = Attribute {
            stage,
            attribute_name_in_rust,
            attribute_name_in_string,
            attribute_value: *expr_assign.right,
            is_event_attribute,
            spair_store_index: 0,
        };

        if attribute.is_event_attribute
        // && attribute.stage == Stage::Creation
        {
            self.events.push(attribute);
        } else if html_tag_in_string == "select" && attribute.attribute_name_in_string == "value" {
            self.select_value_setting = Some(attribute);
        } else {
            self.attributes.push(attribute);
        }
    }

    pub fn validate_html(&self, errors: &mut MultiErrors) {
        self.check_html_tag(errors);
        if self
            .attributes
            .iter()
            .any(|v| v.attribute_name_in_string.as_str() == INNER_HTML)
        {
            if self.children.items.is_empty().not() {
                errors.error_at(
                    self.html_tag_ident.span(),
                    "Setting inner HTML on element that has children is not supported",
                );
            }
        }
        for attribute in self
            .attributes
            .iter()
            .chain(self.select_value_setting.iter())
            .chain(self.events.iter())
        {
            attribute.check_html(&self.html_tag_ident.to_string(), errors);
        }
        for child in self.children.items.iter() {
            child.check_html_multi_errors(errors);
        }
    }

    fn check_html_tag(&self, errors: &mut MultiErrors) {
        match self.html_tag_ident.to_string().as_str() {
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
            _ => {
                errors.error_at(self.html_tag_ident.span(), "unknown html tag")
            },
        }
    }

    pub fn prepare_items_for_generating_code(&mut self) {
        let me_has_only_one_child = self.children.items.len() == 1;
        let mut child_need_parent = false;
        for ittem in self.children.items.iter_mut() {
            ittem.prepare_items_for_generating_code(me_has_only_one_child);
            if child_need_parent.not() {
                child_need_parent = match ittem {
                    Item::Text(_value) => false,
                    Item::Element(_value) => false,
                    Item::View(value) => value.has_update_call(),
                    Item::List(_value) => false,
                    Item::Match(_value) => true,
                    Item::CompRef(_value) => true,
                }
            };
        }
        self.need_this_as_a_parent_for_updating_children = child_need_parent;
        self.count_spair_element_capacity();
    }

    fn count_spair_element_capacity(&mut self) {
        for (index, event) in self.events.iter_mut().enumerate() {
            event.spair_store_index = index;
        }
        let mut store_index = self.events.len();
        for attribute in self
            .attributes
            .iter_mut()
            .chain(self.select_value_setting.iter_mut())
            .filter(|attribute| {
                attribute.is_event_attribute || matches!(&attribute.stage, Stage::Update)
            })
        {
            attribute.spair_store_index = store_index;
            store_index += 1;
        }

        self.spair_element_capacity = store_index;
    }

    pub fn generate_view_state_struct_fields(&self, sub_mod: &SubMod) -> TokenStream {
        let ident = &self.spair_ident;
        let field_for_self = if self.at_root || self.spair_element_capacity > 0 {
            quote! {pub #ident: ::spair::Element, }
        } else if self.need_this_as_a_parent_for_updating_children {
            quote! {pub #ident: ::spair::WsElement, }
        } else {
            quote! {}
        };
        let fields_for_children: TokenStream = self
            .children
            .items
            .iter()
            .map(|v| v.generate_view_state_struct_fields(sub_mod))
            .collect();
        quote! {
            #field_for_self
            #fields_for_children
        }
    }

    pub fn generate_view_states_for_matches_and_lists(&self) -> TokenStream {
        self.children.generate_view_states_for_matches_and_lists()
    }

    pub fn generate_html_string(&self, html_string: &mut String) {
        let html_tag = self.html_tag_ident.to_string();
        let (open_closing, close_1, close_2, close_3) = match html_tag.as_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "source" | "track" | "wbr" => (" />", "", "", ""),
            html_tag => (">", "</", html_tag, ">"),
        };
        html_string.push('<');
        html_string.push_str(&html_tag);
        self.generate_html_string_attributes(html_string);
        html_string.push_str(open_closing);
        self.children.generate_html_string(html_string);
        html_string.push_str(close_1);
        html_string.push_str(close_2);
        html_string.push_str(close_3);
    }

    fn generate_html_string_attributes(&self, html_string: &mut String) {
        for attribute in self
            .attributes
            .iter()
            .chain(self.select_value_setting.iter())
        {
            attribute.generate_html_string(html_string);
        }
    }

    pub fn generate_fn_create(&self, sub_mod: &SubMod, last_node: &LastNode) -> TokenStream {
        let element = &self.spair_ident;
        let get_ws_element = last_node.get_ws_element(&self.spair_ident);
        let get_element = if self.at_root || self.spair_element_capacity > 0 {
            let capacity = self.spair_element_capacity;
            quote! {
                #get_ws_element
                let mut #element = #element.create_element_with_capacity(#capacity);
            }
        } else {
            get_ws_element
        };
        let attribute_setting = self.generate_fn_create_for_attributes();
        let children_creation = self.generate_fn_create_for_children(sub_mod);

        quote! {
            #get_element
            #attribute_setting
            #children_creation
        }
    }

    fn generate_fn_create_for_attributes(&self) -> TokenStream {
        let element = &self.spair_ident;
        let element_name = self.html_tag_ident.to_string();

        self.events
            .iter()
            .chain(self.attributes.iter())
            .chain(self.select_value_setting.iter())
            .map(|v| v.generate_fn_create_element_creation_code(&element_name, element))
            .collect()
    }

    fn generate_fn_create_for_children(&self, sub_mod: &SubMod) -> TokenStream {
        let mut previous = None;
        let mut last_node = LastNode {
            parent: self.spair_ident.clone(),
            previous: None,
        };
        self.children
            .items
            .iter()
            .map(|v| {
                let code = v.generate_fn_create(sub_mod, &mut last_node);
                previous = Some(v.spair_ident_to_get_next_node());
                last_node.previous = previous.cloned();
                code
            })
            .collect()
    }

    pub fn spair_indent_to_get_next_node(&self) -> &Ident {
        &self.spair_ident
    }

    pub fn generate_fn_create_return_value(&self) -> TokenStream {
        let ident = &self.spair_ident;
        let self_element = if self.at_root
            || self.need_this_as_a_parent_for_updating_children
            || self.spair_element_capacity > 0
        {
            quote! {#ident,}
        } else {
            quote! {}
        };
        let view_state_fields: TokenStream = self
            .children
            .items
            .iter()
            .map(|v| v.generate_fn_create_return_value())
            .collect();
        quote! {
            #self_element
            #view_state_fields
        }
    }

    pub fn generate_fn_update(&self, sub_mod: &SubMod, view_state: &Ident) -> TokenStream {
        let attribute_setting = self.generate_fn_update_for_attributes(view_state);
        let setting_select_value = self.generate_fn_update_for_select_element(view_state);
        let children = self.generate_fn_update_for_children(sub_mod, view_state);
        quote! {
            #attribute_setting
            #children
            #setting_select_value
        }
    }

    fn generate_fn_update_for_attributes(&self, view_state: &Ident) -> TokenStream {
        let element = &self.spair_ident;

        let element = quote! {#view_state.#element};
        let element_name = self.html_tag_ident.to_string();

        self.events
            .iter()
            .chain(self.attributes.iter())
            .map(|v| v.generate_view_state_fn_update_code(&element_name, &element))
            .collect()
    }

    fn generate_fn_update_for_select_element(&self, view_state: &Ident) -> TokenStream {
        let element = &self.spair_ident;

        let element = quote! {#view_state.#element};
        let element_name = self.html_tag_ident.to_string();
        self.select_value_setting
            .iter()
            .map(|v| v.generate_view_state_fn_update_code(&element_name, &element))
            .collect()
    }

    fn generate_fn_update_for_children(&self, sub_mod: &SubMod, view_state: &Ident) -> TokenStream {
        self.children
            .items
            .iter()
            .map(|v| v.generate_fn_update(sub_mod, view_state, &self.spair_ident))
            .collect()
    }

    pub fn generate_fn_remove_from(&self, parent: &Ident) -> TokenStream {
        let element = &self.spair_ident;
        quote! {
            #parent.remove_child(&self.#element);
        }
    }
}

fn is_html_event_name(event_name: &str, element_name: &str) -> bool {
    match event_name{
            "input_string" |
            "input_checked" => element_name == "input",
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
