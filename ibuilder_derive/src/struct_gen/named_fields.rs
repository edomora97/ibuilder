use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::Ident;

use quote::{quote, TokenStreamExt};

use crate::struct_gen::StructGenerator;

/// The generator of the implementation of `BuildableValue` for a struct with named fields.
pub struct StructWithNamedFields<'s> {
    /// The base struct generator.
    gen: &'s StructGenerator,
    /// The list of the names of the fields of the original structure.
    fields: Vec<Ident>,
}

impl<'s> StructWithNamedFields<'s> {
    /// Make a new `StructWithNamedFields` from a `StructGenerator`.
    pub fn new(gen: &'s StructGenerator) -> Self {
        if gen.fields.is_empty() {
            abort!(gen.span, "the struct must have at least one field");
        }
        Self {
            fields: gen
                .fields
                .iter()
                .filter(|f| !f.metadata.hidden)
                .filter_map(|f| f.ident.clone())
                .collect(),
            gen,
        }
    }

    /// Generate the implementation of the trait methods.
    pub fn gen(&self) -> TokenStream {
        let fn_apply = self.gen_fn_apply();
        let fn_get_options = self.gen_fn_get_options();
        let fn_get_subfields = self.gen_fn_get_subfields();
        let fn_to_node = self.gen_fn_to_node();
        let fn_get_value_any = self.gen_fn_get_value_any();
        quote! {
            #fn_apply
            #fn_get_options
            #fn_get_subfields
            #fn_to_node
            #fn_get_value_any
        }
    }

    /// Generate the implementation of the `apply` method.
    fn gen_fn_apply(&self) -> TokenStream {
        let field_names = &self.fields;
        quote! {
            fn apply(&mut self, data: ibuilder::Input, current_fields: &[String]) -> Result<(), ibuilder::ChooseError> {
                if current_fields.is_empty() {
                    match data {
                        ibuilder::Input::Choice(data) => {
                            match data.as_str() {
                                #(stringify!(#field_names) => {},)*
                                _ => return Err(ibuilder::ChooseError::UnexpectedChoice),
                            }
                        }
                        _ => return Err(ibuilder::ChooseError::UnexpectedText)
                    }
                } else {
                    let field = &current_fields[0];
                    let rest = &current_fields[1..];
                    match field.as_str() {
                        #(stringify!(#field_names) => self.#field_names.apply(data, rest)?,)*
                        _ => Err(ibuilder::ChooseError::UnexpectedChoice)?,
                    }
                }
                Ok(())
            }
        }
    }

    /// Generate the implementation of the `get_options` method.
    fn gen_fn_get_options(&self) -> TokenStream {
        let field_names = &self.fields;
        let choices = self
            .gen
            .fields
            .iter()
            .filter(|f| !f.metadata.hidden)
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let name = f.actual_name();
                quote! {
                    ibuilder::Choice {
                        choice_id: stringify!(#ident).to_string(),
                        text: "Edit ".to_string() + #name,
                        needs_action: self.#ident.get_value_any().is_none(),
                    }
                }
            });
        quote! {
            fn get_options(&self, current_fields: &[String]) -> ibuilder::Options {
                if current_fields.is_empty() {
                    ibuilder::Options {
                        query: self.__prompt.clone(),
                        text_input: false,
                        choices: vec![ #(#choices),* ],
                    }
                } else {
                    let field = &current_fields[0];
                    let rest = &current_fields[1..];
                    match field.as_str() {
                        #(stringify!(#field_names) => self.#field_names.get_options(rest),)*
                        _ => unreachable!("Invalid current field: {} (the rest is {:?})", field, rest),
                    }
                }
            }
        }
    }

    /// Generate the implementation of the `get_subfields` method.
    fn gen_fn_get_subfields(&self) -> TokenStream {
        let field_names = &self.fields;
        quote! {
            fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
                if current_fields.is_empty() {
                    vec![ #(stringify!(#field_names).to_string(),)* ]
                } else {
                    let field = &current_fields[0];
                    let rest = &current_fields[1..];
                    match field.as_str() {
                        #(stringify!(#field_names) => self.#field_names.get_subfields(rest),)*
                        _ => unreachable!("Invalid current field: {} (the rest is {:?})", field, rest),
                    }
                }
            }
        }
    }

    /// Generate the implementation of the `to_node` method.
    fn gen_fn_to_node(&self) -> TokenStream {
        let ident = &self.gen.ident;
        let fields: Vec<_> = self
            .gen
            .fields
            .iter()
            .filter(|f| !f.metadata.hidden)
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let name = f.actual_name();
                quote! {
                    ibuilder::nodes::FieldKind::Named(#name.into(), self.#ident.to_node())
                }
            })
            .collect();
        let name = if let Some(name) = &self.gen.metadata.rename {
            quote! { #name }
        } else {
            quote! { stringify!(#ident) }
        };
        quote! {
            fn to_node(&self) -> ibuilder::nodes::Node {
                ibuilder::nodes::Node::Composite(
                    #name.into(),
                    vec![ #(#fields,)* ]
                )
            }
        }
    }

    /// Generate the implementation of the `get_value_any` method.
    fn gen_fn_get_value_any(&self) -> TokenStream {
        let ident = &self.gen.ident;
        let mut field_list = TokenStream::new();
        for field in self.gen.fields.iter() {
            let field_name = field.ident.as_ref().unwrap();
            field_list.append_all(if field.metadata.hidden {
                quote! { #field_name: self.#field_name.clone(), }
            } else {
                quote! { #field_name: *self.#field_name.get_value_any()?.downcast().unwrap(), }
            });
        }
        quote! {
            fn get_value_any(&self) -> Option<Box<dyn std::any::Any>> {
                Some(Box::new(#ident {
                    #field_list
                }))
            }
        }
    }
}
