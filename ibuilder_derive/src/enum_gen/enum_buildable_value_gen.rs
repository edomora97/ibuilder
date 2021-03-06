use proc_macro2::TokenStream;
use quote::quote;

use crate::enum_gen::{
    gen_variants_builder_ident, gen_variants_builder_variant_ident, EnumGenerator, VariantKind,
};

/// Generate the implementation of the `BuildableValue` trait.
pub fn gen_impl_buildable_value(gen: &EnumGenerator) -> TokenStream {
    let builder_ident = &gen.builder_ident;
    let fn_apply = gen_fn_apply(gen);
    let fn_get_options = gen_fn_get_options(gen);
    let fn_get_subfields = gen_fn_get_subfields(gen);
    let fn_to_node = gen_fn_to_node(gen);
    let fn_get_value_any = gen_fn_get_value_any(gen);
    quote! {
        #[automatically_derived]
        #[allow(unreachable_code)]
        impl ibuilder::BuildableValue for #builder_ident {
            #fn_apply
            #fn_get_options
            #fn_get_subfields
            #fn_to_node
            #fn_get_value_any
        }
    }
}

/// Generate the implementation of the `apply` method.
///
/// If the builder is in the variant menu, apply selects the variant to use. If it is already inside
/// a variant it forward the apply to it.
fn gen_fn_apply(gen: &EnumGenerator) -> TokenStream {
    let select_menu = fn_apply_select_menu(gen);
    let inner_menu = fn_apply_inner_menu(gen);
    quote! {
        fn apply(&mut self, data: ibuilder::Input, current_fields: &[String]) -> Result<(), ibuilder::ChooseError> {
            // select variant menu
            if current_fields.is_empty() {
                #select_menu
            } else {
                #inner_menu
            }
            Ok(())
        }
    }
}

/// Generate the selection of the variant to use.
fn fn_apply_select_menu(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let select_menu: Vec<_> = gen
        .variants
        .iter()
        .filter(|v| !v.metadata.hidden)
        .map(|var| {
            let ident = &var.ident;
            let variant_builder_new = var.builder_new(&gen.ident);
            let content = if var.kind.is_empty() {
                quote! {}
            } else {
                quote! {(_)}
            };
            quote! {
                stringify!(#ident) => {
                    match &self.value {
                        // do not overwrite if already selected
                        Some(#builder::#ident #content) => {},
                        _ => self.value = Some(#variant_builder_new)
                    }
                }
            }
        })
        .collect();
    quote! {
        match data {
            ibuilder::Input::Choice(data) => {
                match data.as_str() {
                    #(#select_menu,)*
                    _ => return Err(ibuilder::ChooseError::UnexpectedChoice),
                }
            }
            _ => return Err(ibuilder::ChooseError::UnexpectedText)
        }
    }
}

/// Generate the forwarding of the apply to the variant.
fn fn_apply_inner_menu(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let apply: Vec<_> = gen
        .variants
        .iter()
        .filter(|v| !v.metadata.hidden)
        .filter_map(|var| match &var.kind {
            VariantKind::Empty => None,
            VariantKind::Unnamed(_) | VariantKind::Named(_) => {
                let variant = &var.ident;
                Some(quote! {
                    stringify!(#variant) => match self.value.as_mut().unwrap() {
                        #builder::#variant(inner) => inner.apply(data, rest)?,
                        _ => unreachable!("Invalid variant in value"),
                    }
                })
            }
        })
        .collect();
    quote! {
        let field = &current_fields[0];
        let rest = &current_fields[1..];
        match field.as_str() {
            #(#apply,)*
            _ => unreachable!("Invalid variant: {}", field),
        }
    }
}

/// Generate the implementation of the `get_options` method.
///
/// If the builder is in the main menu, allow the selection of one of the variants. If already
/// inside a variant forwards the call to it.
fn gen_fn_get_options(gen: &EnumGenerator) -> TokenStream {
    let select_menu = fn_get_options_select_menu(gen);
    let inner_menu = fn_get_options_inner_menu(gen);
    quote! {
        fn get_options(&self, current_fields: &[String]) -> ibuilder::Options {
            if current_fields.is_empty() {
                #select_menu
            } else {
                #inner_menu
            }
        }
    }
}

/// Generate the menu of selection of the variant.
fn fn_get_options_select_menu(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let choices: Vec<_> = gen
        .variants
        .iter()
        .filter(|v| !v.metadata.hidden)
        .map(|var| {
            let ident = &var.ident;
            let name = var.actual_name();
            let needs_action = match &var.kind {
                // empty variants never need actions
                VariantKind::Empty => quote! { false },
                VariantKind::Unnamed(_) | VariantKind::Named(_) => {
                    quote! {
                        match self.value.as_ref() {
                            Some(#builder::#ident(inner)) => inner.get_value_any().is_none(),
                            _ => false,
                        }
                    }
                }
            };
            quote! {
                ibuilder::Choice {
                    choice_id: stringify!(#ident).to_string(),
                    text: #name.to_string(),
                    needs_action: #needs_action,
                }
            }
        })
        .collect();
    quote! {
        ibuilder::Options {
            query: self.prompt.clone(),
            text_input: false,
            choices: vec![ #(#choices,)* ],
        }
    }
}

/// Generate the forwarding of the call to get_options to the variant.
fn fn_get_options_inner_menu(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let variants: Vec<_> = gen
        .variants
        .iter()
        .filter(|v| !v.metadata.hidden)
        .filter_map(|var| {
            let ident = &var.ident;
            match &var.kind {
                VariantKind::Empty => None,
                VariantKind::Unnamed(_) | VariantKind::Named(_) => Some(quote! {
                    stringify!(#ident) => match self.value.as_ref().unwrap() {
                        #builder::#ident(inner) => inner.get_options(rest),
                        _ => unreachable!("Invalid variant in value"),
                    }
                }),
            }
        })
        .collect();
    quote! {
        let field = &current_fields[0];
        let rest = &current_fields[1..];
        match field.as_str() {
            #(#variants,)*
            _ => unreachable!("Invalid variant {}", field),
        }
    }
}

/// Generate the implementation of the `get_subfields` method.
fn gen_fn_get_subfields(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let variants: Vec<_> = gen
        .variants
        .iter()
        .filter(|var| !var.kind.is_empty() && !var.metadata.hidden)
        .map(|var| &var.ident)
        .collect();
    quote! {
        fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
            if current_fields.is_empty() {
                vec![ #(stringify!(#variants).to_string(),)* ]
            } else {
                let field = &current_fields[0];
                let rest = &current_fields[1..];
                match field.as_str() {
                    #(
                        stringify!(#variants) => match self.value.as_ref().unwrap() {
                            #builder::#variants(inner) => inner.get_subfields(rest),
                            _ => unreachable!("Invalid variant in value"),
                        },
                    )*
                    _ => unreachable!("Invalid variant: {}", field),
                }
            }
        }
    }
}

/// Generate the implementation of the `to_node` method.
fn gen_fn_to_node(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let variants: Vec<_> = gen
        .variants
        .iter()
        .filter(|v| !v.metadata.hidden)
        .map(|var| {
            let ident = &var.ident;
            let name = var.actual_name();
            match &var.kind {
                VariantKind::Empty => quote! {
                    Some(#builder::#ident) => {
                        ibuilder::nodes::Node::Leaf(ibuilder::nodes::Field::String(#name.to_string()))
                    }
                },
                VariantKind::Named(_) => quote! {
                    Some(#builder::#ident(inner)) => {
                        let inner_node = inner.to_node();
                        let fields = match inner_node {
                            ibuilder::nodes::Node::Composite(_, fields) => fields,
                            _ => unreachable!("Invalid node of enum content"),
                        };
                        ibuilder::nodes::Node::Composite(#name.to_string(), fields)
                    }
                },
                VariantKind::Unnamed(_) => quote! {
                    Some(#builder::#ident(inner)) => inner.to_node()
                },
            }
        })
        .collect();
    quote! {
        fn to_node(&self) -> ibuilder::nodes::Node {
            match &self.value {
                None => ibuilder::nodes::Node::Leaf(ibuilder::nodes::Field::Missing),
                #(#variants,)*
                _ => unreachable!("Selected an hidden value")
            }
        }
    }
}

/// Generate the implementation of the `get_value_any` method.
fn gen_fn_get_value_any(gen: &EnumGenerator) -> TokenStream {
    let builder = gen_variants_builder_ident(&gen.ident);
    let base = &gen.ident;
    let variants: Vec<_> = gen
        .variants
        .iter()
        .map(|var| {
            let ident = &var.ident;
            match &var.kind {
                VariantKind::Empty => quote! { #builder::#ident => Box::new(#base::#ident) },
                VariantKind::Named(_) => {
                    let fields = var.field_names();
                    let field_builder = gen_variants_builder_variant_ident(&gen.ident, ident);
                    quote! {
                        #builder::#ident(inner) => {
                            let inner = inner
                                .get_value_any()?
                                .downcast::<#field_builder>()
                                .unwrap();
                            Box::new(#base::#ident {
                                #(#fields: inner.#fields,)*
                            })
                        }
                    }
                }
                VariantKind::Unnamed(fields) => {
                    let fields = (0..fields.len()).map(syn::Index::from);
                    let field_builder = gen_variants_builder_variant_ident(&gen.ident, ident);
                    quote! {
                        #builder::#ident(inner) => {
                            let inner = inner
                                .get_value_any()?
                                .downcast::<#field_builder>()
                                .unwrap();
                            Box::new(#base::#ident(#(inner.#fields,)*))
                        }
                    }
                }
            }
        })
        .collect();
    quote! {
        fn get_value_any(&self) -> Option<Box<dyn std::any::Any>> {
            let variant = self.value.as_ref()?;
            Some(match variant {
                #(#variants,)*
            })
        }
    }
}
