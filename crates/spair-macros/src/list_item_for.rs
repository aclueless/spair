use proc_macro2::{Span, TokenStream};
use quote::quote;
use spanned::Spanned;
use syn::*;

use crate::{element::HtmlElement, new_view::View};

pub struct ListItemView {
    list_item_type_name: Ident,
    component_type_name: Ident,
    item_impl: ItemImpl,
    element: HtmlElement,
}

impl ListItemView {
    pub fn with_item_impl(item_impl: ItemImpl) -> Result<Self> {
        if item_impl.items.is_empty() {
            return Err(Error::new(
                item_impl.span(),
                "Expected 3 methods: `create_view`, `update_view`, `view`",
            ));
        }
        let View {
            view_state_type_name,
            item_impl,
            element,
        } = crate::new_view::View::with_item_impl(item_impl)?;

        let component_type_name = get_component_type_name(&item_impl)?;

        Ok(Self {
            list_item_type_name: view_state_type_name,
            component_type_name,
            item_impl,
            element,
        })
    }

    pub fn output(&self) -> TokenStream {
        let list_item_view_state_struct_name = Ident::new(
            &format!("{}ViewState", self.list_item_type_name),
            Span::call_site(),
        );
        let struct_fields = self.element.generate_view_state_struct_fields();
        let view_state_struct = quote! {pub struct #list_item_view_state_struct_name{
            #struct_fields
        }};
        let match_view_state_types = self.element.collect_match_view_state_types();
        let impl_list_item_view =
            self.generate_impl_list_item_view(&list_item_view_state_struct_name);

        quote! {
            #view_state_struct
            #match_view_state_types
            #impl_list_item_view
        }
    }

    fn generate_impl_list_item_view(
        &self,
        keyed_item_view_state_struct_name: &Ident,
    ) -> TokenStream {
        let generated_create_view_fn =
            self.generate_impl_create_view_fn(keyed_item_view_state_struct_name);
        let generated_update_view_fn = self.generate_impl_update_view_fn();

        let impl_token = &self.item_impl.impl_token;
        let self_type = &self.item_impl.self_ty;
        let component_type = &self.component_type_name;
        let root_element_ident = self.element.root_element_ident();
        let html_string = self.element.construct_html_string();
        quote! {
            #impl_token ::spair::ListItemView<#component_type> for #self_type {
                type ViewState = #keyed_item_view_state_struct_name;
                fn root_element(view_state: &Self::ViewState) -> &::spair::WsElement {
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
        let impl_item = self.item_impl.items.first();
        let ImplItem::Fn(ImplItemFn { sig, block, .. }) = impl_item.unwrap() else {
            unreachable!("There must be an fn")
        };
        let Signature {
            fn_token,
            ident,
            inputs,
            ..
        } = sig;
        let fn_body = self
            .element
            .generate_code_for_create_view_fn_of_a_keyed_item_view(
                view_state_struct_name,
                quote! {},
            );
        let block = &block.stmts;
        quote! {
            #fn_token #ident(_keyed_view_state_template: &::spair::TemplateElement, #inputs) -> Self::ViewState {
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
