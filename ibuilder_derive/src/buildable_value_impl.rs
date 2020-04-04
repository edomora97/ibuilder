use syn::export::TokenStream2;
use syn::{DataStruct, Ident};

use quote::quote;

use crate::field_data::field_initializers;

/// Generate the `TokenStream` of the structure definition for a new structure that implements
/// `BuildableValue`.
///
/// ```
/// struct Example {
///     field1: i64,
///     field2: String,
/// }
/// ```
///
/// will generate
///
/// ```
/// # extern crate ibuilder;
/// #[automatically_derived]
/// #[allow(non_camel_case_types)]
/// #[doc(hidden)]
/// #[derive(Debug)]
/// struct __Example_BuildableValueImpl {
///     __help: String,
///     field1: Box<dyn ibuilder::BuildableValue>, // actually an I64Builder
///     field2: Box<dyn ibuilder::BuildableValue>, // actually an StringBuilder
/// }
/// ```
pub fn gen_struct_buildable_value(data: &DataStruct, buildable_value_name: &Ident) -> TokenStream2 {
    let mut builder_impl_fields = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        builder_impl_fields.push(quote! {
            #field_name: Box<dyn ibuilder::BuildableValue>
        });
    }
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(Debug)]
        struct #buildable_value_name {
            __help: String,
            #(#builder_impl_fields,)*
        }
    }
}

/// Generate the implementation of the `new` method for the structure.
///
/// For example it can generate something like:
///
/// ```
/// # struct __Example_BuildableValueImpl;
/// # struct Example;
/// impl __Example_BuildableValueImpl {
///     pub fn new(help: String) -> Self {
///         # unimplemented!()
///         // ...
///     }
/// }
/// ```
pub fn gen_impl_struct_buildable_value(
    data: &DataStruct,
    buildable_value_name: &Ident,
) -> TokenStream2 {
    let builder_init = field_initializers(data);
    quote! {
        impl #buildable_value_name {
            fn new(help: String) -> Box<dyn ibuilder::BuildableValue> {
                Box::new(Self {
                    __help: help,
                    #builder_init
                })
            }
        }
    }
}

/// Generate the implementation of the `BuildableValue` trait for the specified struct.
pub fn gen_impl_buildable_value(
    data: &DataStruct,
    name: &Ident,
    buildable_value_name: &Ident,
) -> TokenStream2 {
    let fn_apply = gen_fn_apply(data);
    let fn_get_options = gen_fn_get_options(data);
    let fn_get_subfield = gen_fn_get_subfields(data);
    let fn_to_node = gen_fn_to_node(name, data);
    let fn_get_value_any = gen_fn_get_value_any(name, data);
    quote! {
        impl ibuilder::BuildableValue for #buildable_value_name {
            fn get_help(&self) -> &str {
                &self.__help
            }

            #fn_apply
            #fn_get_options
            #fn_get_subfield
            #fn_to_node
            #fn_get_value_any
        }
    }
}

/// Generate the `BuildableValue::apply` function for the structure. Apply is allowed only for the
/// inner fields, so if `current_fields` is empty it means that no field is selected, and this is
/// a bug of the base `Builder`.
///
/// Otherwise, if a field is selected, the action is forwarded to that field.
fn gen_fn_apply(data: &DataStruct) -> TokenStream2 {
    let mut field_apply = Vec::new();
    let mut field_names = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_lit =
            syn::LitStr::new(&format!("{}", field_name), syn::export::Span::call_site());
        field_apply.push(quote! {
            #field_name_lit => self.#field_name.apply(data, rest)?
        });
        field_names.push(field_name_lit);
    }

    quote! {
        fn apply(&mut self, data: &str, current_fields: &[String]) -> Result<(), ibuilder::ChooseError> {
            if current_fields.is_empty() {
                match data {
                    #(#field_names)|* => {},
                    _ => return Err(ibuilder::ChooseError::UnexpectedChoice),
                }
            } else {
                let field = &current_fields[0];
                let rest = &current_fields[1..];
                match field.as_str() {
                    #(#field_apply,)*
                    _ => Err(ibuilder::ChooseError::UnexpectedChoice)?,
                }
            }
            Ok(())
        }
    }
}

/// Generate the `BuildableValue::get_options` function for the structure. When no field is selected
/// the returned options are the ones for selecting the inner field to edit. Otherwise, when a field
/// is selected the options shown come from it.
fn gen_fn_get_options(data: &DataStruct) -> TokenStream2 {
    let mut field_get_options = Vec::new();
    let mut choices = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_lit =
            syn::LitStr::new(&format!("{}", field_name), syn::export::Span::call_site());
        let choice_text = syn::LitStr::new(
            &format!("Edit {}", field_name),
            syn::export::Span::call_site(),
        );
        field_get_options.push(quote! {
            #field_name_lit => self.#field_name.get_options(rest)
        });
        choices.push(quote! {
            ibuilder::Choice {
                choice_id: #field_name_lit.to_string(),
                text: #choice_text.to_string(),
                needs_action: self.#field_name.get_value_any().is_none(),
            }
        });
    }

    quote! {
        fn get_options(&self, current_fields: &[String]) -> ibuilder::Options {
            if current_fields.is_empty() {
                ibuilder::Options {
                    query: "Select the field to edit".to_string(),
                    text_input: false,
                    choices: vec![
                        #(#choices,)*
                    ],
                }
            } else {
                let field = &current_fields[0];
                let rest = &current_fields[1..];
                match field.as_str() {
                    #(#field_get_options,)*
                    _ => panic!("Invalid current field: {} (the rest is {:?})", field, rest),
                }
            }
        }
    }
}

/// Generate the `BuildableValue::get_subfields` function for the structure. When no field is
/// selected it will return the list of the fields for the current structure, otherwise it will
/// forward the request to the selected field.
fn gen_fn_get_subfields(data: &DataStruct) -> TokenStream2 {
    let mut fields = Vec::new();
    let mut field_get_subfields = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_lit =
            syn::LitStr::new(&format!("{}", field_name), syn::export::Span::call_site());
        fields.push(field_name_lit.clone());
        field_get_subfields.push(quote! {
            #field_name_lit => self.#field_name.get_subfields(rest)
        });
    }
    quote! {
        fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
            if current_fields.is_empty() {
                vec![ #(#fields.to_string(),)* ]
            } else {
                let field = &current_fields[0];
                let rest = &current_fields[1..];
                match field.as_str() {
                    #(#field_get_subfields,)*
                    _ => panic!("Invalid current field: {} (the rest is {:?})", field, rest),
                }
            }
        }
    }
}

/// Generate the `BuildableValue::to_node` function for the structure.
pub fn gen_fn_to_node(name: &Ident, data: &DataStruct) -> TokenStream2 {
    let mut fields = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_lit =
            syn::LitStr::new(&format!("{}", field_name), syn::export::Span::call_site());
        fields.push(quote! {
            ibuilder::nodes::FieldKind::Named(#field_name_lit.into(), self.#field_name.to_node())
        });
    }
    quote! {
        fn to_node(&self) -> ibuilder::nodes::Node {
            ibuilder::nodes::Node::Composite(
                stringify!(#name).into(),
                vec![
                    #(#fields,)*
                ]
            )
        }
    }
}

/// Generate the `BuildableValue::get_value_any` function for the structure.
pub fn gen_fn_get_value_any(name: &Ident, data: &DataStruct) -> TokenStream2 {
    let mut get_value_fields = Vec::new();
    for field in data.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        get_value_fields.push(quote! {
            #field_name: *self.#field_name.get_value_any()?.downcast::<#field_type>().unwrap()
        });
    }
    quote! {
        fn get_value_any(&self) -> Option<Box<dyn std::any::Any>> {
            Some(Box::new(#name {
                #(#get_value_fields,)*
            }))
        }
    }
}
