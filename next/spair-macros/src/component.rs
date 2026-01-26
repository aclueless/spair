use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Block, Ident, ImplItem, ItemImpl, Result, ReturnType};

use crate::{
    MultiErrors,
    dom::{Element, Item, LastNode, SubMod},
    view::{View, insert_use_spair_items_to_fn},
};

pub struct Component {
    _component_name: Ident,
    view_state_name: Ident,
    item_impl: ItemImpl,
    element: Element,

    sub_mod: SubMod,
}
impl Component {
    pub fn from_item_impl(item_impl: syn::ItemImpl) -> Result<Self> {
        let view = View::from_item_impl(item_impl)?;
        Component::from_view(view)
    }

    fn from_view(view: View) -> Result<Self> {
        let mut errors = MultiErrors::default();
        let View {
            view_name,
            vis: _,
            item_impl,
            items,

            span_to_report_empty,
            sub_mod,
        } = view;
        let mut items = items.into_inner();
        if items.is_empty() {
            return errors.with_last_error_at(
                span_to_report_empty,
                "Expected an HTML element at root, like: div(...)..., found nothing.",
            );
        }

        let message = "Expected exactly an HTML element at root, like: div(...)..., found ";
        let element = loop {
            if items.is_empty() {
                return errors.with_last_error_at(span_to_report_empty, "No HTML element found");
            }
            match items.remove(0) {
                Item::Text(value) => errors.error_at2(value.first_span(), message, " a text here"),
                Item::Element(value) => break value,
                Item::View(value) => errors.error_at2(value.first_span(), message, "a view here"),
                Item::List(value) => errors.error_at2(value.first_span(), message, "a list here"),
                Item::Match(value) => errors.error_at2(value.first_span(), message, "a match here"),
                Item::CompRef(value) => {
                    errors.error_at2(value.first_span(), message, "a spair_comp_ref here")
                }
            }
        };
        for item in items {
            errors.error_at2(item.first_span(), message, " more item here");
        }
        errors.report_error()?;

        let view_state = Ident::new(&format!("_{view_name}SpairViewState"), view_name.span());
        Ok(Component {
            _component_name: view_name,
            view_state_name: view_state,
            item_impl,
            element,
            sub_mod,
        })
    }

    pub fn generate(&self) -> TokenStream {
        // the struct to store the state of this view
        let view_state_struct_for_self = self.generate_view_state_struct();

        // view for all `match` items in this view's DOM
        let view_states_for_matches_and_lists =
            self.element.generate_view_states_for_matches_and_lists();
        let view_states_for_match_items =
            self.sub_mod.generate_mod(view_states_for_matches_and_lists);

        let mut impl_view_state = self.item_impl.clone();

        // fn create(...) -> Self::ViewState {...}
        self.generate_fn_create(
            impl_view_state
                .items
                .first_mut()
                .expect("get fn create for component"),
        );

        // fn update(view_state: &mut Self::ViewState, ...) {...}
        self.generate_update_fn(
            impl_view_state
                .items
                .last_mut()
                .expect("get fn update for component"),
        );

        // modify impl to make it like:
        // impl ::spair::Component for ComponentName {
        //     type ViewState = ComponentViewState;
        // }
        self.make_it_impl_component(&mut impl_view_state);

        quote! {
            #view_state_struct_for_self
            #view_states_for_match_items
            #impl_view_state
        }
    }

    fn generate_view_state_struct(&self) -> TokenStream {
        let view_state_struct_name = &self.view_state_name;
        let view_state_struct_fields = self
            .element
            .generate_view_state_struct_fields(&self.sub_mod);
        let view_state_root_element = self.element.spair_indent_to_get_next_node();
        quote! {
            pub struct #view_state_struct_name{#view_state_struct_fields}
            impl ::spair::ComponentViewState for #view_state_struct_name{
                fn root_element(&self) -> &::spair::Element {
                    &self.#view_state_root_element
                }
            }
        }
    }

    fn make_it_impl_component(&self, item_impl: &mut ItemImpl) {
        // add ::spair::Component for
        let spair_component = quote! {::spair::Component};
        let spair_component = syn::parse(spair_component.into()).expect("::spair::Component");
        let for_keyword = syn::token::For::default();
        item_impl.trait_ = Some((None, spair_component, for_keyword));

        // add type ViewState = ViewStateName;
        let view_state = &self.view_state_name;
        let impl_item_type = quote! {type ViewState = #view_state;};
        let impl_item_type: syn::ImplItemType = syn::parse(impl_item_type.into())
            .expect("type ViewState = ViewStateName; for component");
        item_impl
            .items
            .insert(0, syn::ImplItem::Type(impl_item_type));
    }

    fn generate_fn_create(&self, fn_create: &mut ImplItem) {
        let ImplItem::Fn(create_fn) = fn_create else {
            return;
        };

        // add return type to make it `fn create(...) -> Self`
        let return_type = quote! {-> Self::ViewState};
        let return_type: ReturnType =
            syn::parse(return_type.into()).expect("-> Self::ViewState for create component");
        create_fn.sig.output = return_type;

        // insert `use ::spair::*` to the beginning of the fn create body
        insert_use_spair_items_to_fn(create_fn);

        // add create_fn_code to the end of the fn create body
        let fn_create_fn_body = self.generate_fn_create_fn_body();
        let fn_create_stmts: Block =
            syn::parse(fn_create_fn_body.into()).expect("fn create fn body for create component");
        create_fn.block.stmts.extend(fn_create_stmts.stmts);
    }

    fn generate_fn_create_fn_body(&self) -> TokenStream {
        let mut html_static_string = String::new();
        self.element.generate_html_string(&mut html_static_string);

        let template_fragment = Ident::new("_spair_view_document_fragment_", Span::call_site());
        let last_node = LastNode {
            parent: template_fragment.clone(),
            previous: None,
        };

        let create_elements = self.element.generate_fn_create(&self.sub_mod, &last_node);
        let view_state_instance = self.generate_fn_create_return_value();
        let create_fn_code = quote! {{
            const HTML_STRING: &str = #html_static_string;
            let #template_fragment = ::spair::TemplateElement::new(HTML_STRING).fragment();
            #create_elements
            #view_state_instance
        }};
        create_fn_code
    }

    fn generate_fn_create_return_value(&self) -> TokenStream {
        let fields = self.element.generate_fn_create_return_value();
        let name = &self.view_state_name;
        quote! {
            #name {
                #fields
            }
        }
    }

    fn generate_update_fn(&self, fn_update: &mut ImplItem) {
        let ImplItem::Fn(fn_update) = fn_update else {
            return;
        };

        let view_state = Ident::new("_spair_self_this_me_view_state_", Span::call_site());

        // insert `use ::spair::*` to the beginning of the fn create body
        insert_use_spair_items_to_fn(fn_update);

        // insert `&mut self` to make it `fn update(&mut self, ...)`
        let view_state_name = &self.view_state_name;
        let view_state_arg = quote! {#view_state: &mut #view_state_name};
        let view_state_arg: syn::FnArg = syn::parse(view_state_arg.into())
            .expect("view_state: &mut ViewStateName for update component");
        fn_update.sig.inputs.insert(0, view_state_arg);

        let update_code = self.element.generate_fn_update(&self.sub_mod, &view_state);
        let update_code = quote! {{
            #update_code
        }};
        let update_code: Block =
            syn::parse(update_code.into()).expect("fn update code for component");
        fn_update.block.stmts.extend(update_code.stmts);
    }
}
