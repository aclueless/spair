use std::ops::Not;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use spanned::Spanned;
use syn::*;

use crate::{element::HtmlElement, view::View, MultiErrors};

pub struct KeyedItemView {
    keyed_item_type_name: Ident,
    component_type_name: Ident,
    key_type_name: Ident,
    get_key_fn: ImplItemFn,
    item_impl: ItemImpl,
    element: HtmlElement,
}

impl KeyedItemView {
    pub fn with_item_impl(mut item_impl: ItemImpl) -> Result<Self> {
        if item_impl.items.is_empty() {
            return Err(Error::new(
                item_impl.span(),
                "Expected 4 methods: `get_key`, `create_view`, `update_view`, `view`",
            ));
        }
        let get_key_fn = match item_impl.items.remove(0) {
            ImplItem::Fn(get_key_fn) => get_key_fn,
            other_item => {
                return Err(Error::new(
                    other_item.span(),
                    "Expected `fn get_key(&self) -> &SomeKeyType {...}`",
                ));
            }
        };
        validate_get_fn(&get_key_fn, "get_key")?;
        let View {
            view_state_type_name,
            item_impl,
            element,
        } = crate::view::View::with_item_impl(item_impl)?;

        let key_type_name = get_key_type_name(&get_key_fn)?;
        let component_type_name = get_component_type_name(&item_impl)?;

        Ok(Self {
            keyed_item_type_name: view_state_type_name,
            component_type_name,
            key_type_name,
            get_key_fn,
            item_impl,
            element,
        })
    }

    pub fn output(&self) -> TokenStream {
        let keyed_item_view_state_struct_name = Ident::new(
            &format!("{}ViewState", self.keyed_item_type_name),
            Span::call_site(),
        );
        let struct_fields = self.element.generate_view_state_struct_fields();
        let key_type = &self.key_type_name;
        let view_state_struct = quote! {pub struct #keyed_item_view_state_struct_name{
            key: #key_type,
            #struct_fields
        }};
        let match_view_state_types = self.element.collect_match_view_state_types();
        let impl_keyed_item_view =
            self.generate_impl_keyed_item_view(&keyed_item_view_state_struct_name);

        quote! {
            #view_state_struct
            #match_view_state_types
            #impl_keyed_item_view
        }
    }

    fn generate_impl_keyed_item_view(
        &self,
        keyed_item_view_state_struct_name: &Ident,
    ) -> TokenStream {
        let generated_create_view_fn =
            self.generate_impl_create_view_fn(keyed_item_view_state_struct_name);
        let generated_update_view_fn = self.generate_impl_update_view_fn();

        let impl_token = &self.item_impl.impl_token;
        let self_type = &self.item_impl.self_ty;
        let component_type = &self.component_type_name;
        let key_type = &self.key_type_name;
        let get_key = &self.get_key_fn;
        let root_element_ident = self.element.root_element_ident();
        let html_string = self.element.construct_html_string();
        quote! {
            #impl_token KeyedItemView<#component_type> for #self_type {
                type ViewState = #keyed_item_view_state_struct_name;
                type Key = #key_type;
                #get_key
                fn key_from_view_state(view_state: &Self::ViewState) -> &Self::Key {
                    &view_state.key
                }
                fn root_element(view_state: &Self::ViewState) -> &WsElement {
                    &view_state.#root_element_ident
                }
                fn template_string() -> &'static str {
                    #html_string
                }
                #generated_create_view_fn
                #generated_update_view_fn
            }
        }
    }

    fn generate_impl_create_view_fn(&self, view_state_struct_name: &Ident) -> TokenStream {
        let impl_item = self.item_impl.items.get(0);
        let ImplItem::Fn(ImplItemFn { sig, block, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        let Signature {
            fn_token,
            ident,
            inputs,
            ..
        } = sig;
        let key_field = match inputs.first() {
            Some(FnArg::Typed(pt)) => match pt.pat.as_ref() {
                Pat::Ident(pat_ident) => quote! {key: #pat_ident.get_key().clone(),},
                _ => quote! {},
            },
            _ => quote! {},
        };
        let fn_body = self
            .element
            .generate_code_for_create_view_fn_of_a_keyed_item_view(
                view_state_struct_name,
                key_field,
            );
        let block = &block.stmts;
        quote! {
            #fn_token #ident(_keyed_view_state_template: &TemplateElement, #inputs) -> Self::ViewState {
                #(#block)*
                #fn_body
            }
        }
    }

    fn generate_impl_update_view_fn(&self) -> TokenStream {
        let view_state_ident = Ident::new(
            "_spair_keyed_item_view_state_for_updating_",
            Span::call_site(),
        );
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
            #fn_token #ident(#view_state_ident: &mut Self::ViewState, #inputs) {
                #(#block)*
                #fn_body
            }
        }
    }
}

fn get_key_type_name(get_key_fn: &ImplItemFn) -> Result<Ident> {
    const EM: &str = "Expected a return type like `&SomeTypeName`";
    let rt = match &get_key_fn.sig.output {
        ReturnType::Default => {
            return Err(Error::new(get_key_fn.sig.fn_token.span(), EM));
        }
        ReturnType::Type(_, rt) => rt,
    };
    let rt = match rt.as_ref() {
        Type::Reference(type_reference) => type_reference,
        rt => return Err(Error::new(rt.span(), EM)),
    };
    let ident = match rt.elem.as_ref() {
        Type::Path(type_path) => type_path.path.require_ident()?,
        rt => return Err(Error::new(rt.span(), EM)),
    };

    Ok(ident.clone())
}

fn get_component_type_name(item_impl: &ItemImpl) -> Result<Ident> {
    let cview_fn = item_impl.items.first().unwrap();
    let ImplItem::Fn(cview_fn) = cview_fn else {
        unreachable!("Guarantee by view::View::with_item_impl");
    };
    let Some(FnArg::Typed(context)) = cview_fn.sig.inputs.get(1) else {
        return Err(Error::new(
            cview_fn.sig.inputs.span(),
            "The SECOND ARG must be `ccontext: &Context<ComponentType>`",
        ));
    };
    let message = "Expected: `&Context<ComponentType>`";
    let context = match context.ty.as_ref() {
        Type::Reference(type_reference) => type_reference,
        other => return Err(Error::new(other.span(), message)),
    };
    let type_path = match context.elem.as_ref() {
        Type::Path(type_path) => type_path,
        other => {
            return Err(Error::new(other.span(), message));
        }
    };
    if type_path.path.segments.len() != 1 {
        return Err(Error::new(type_path.span(), message));
    }
    let type_name = type_path.path.segments.first().unwrap();
    let message = "Expected a single identifier as a component type name here like in `Context<ComponentType>`";
    let args = match &type_name.arguments {
        PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
            &angle_bracketed_generic_arguments.args
        }
        other => return Err(Error::new(other.span(), message)),
    };
    if args.len() != 1 {
        return Err(Error::new(args.span(), message));
    }
    let arg = args.first().unwrap();
    let ident = match arg {
        GenericArgument::Type(Type::Path(arg_type)) => arg_type.path.require_ident()?,
        other => return Err(Error::new(other.span(), message)),
    };
    Ok(ident.clone())
}

fn validate_get_fn(item_fn: &ImplItemFn, expected_name: &str) -> Result<()> {
    let mut errors = MultiErrors::default();
    let pub_fn_expected_name_message = format!("Expected `fn {expected_name}(...) `");
    if item_fn.sig.ident != expected_name {
        errors.add(item_fn.sig.ident.span(), &pub_fn_expected_name_message);
    }
    if matches!(&item_fn.vis, Visibility::Inherited).not() {
        errors.add(
            item_fn.sig.span(),
            "Do not give `pub` here, the macro will add it if it is required",
        );
    }
    if let Some(_) = item_fn.sig.generics.lt_token.as_ref() {
        errors.add(
            item_fn.sig.generics.span(),
            "Spair does not support generics",
        );
    }
    errors.report_error()
}
