use proc_macro_error::abort;
use syn::export::TokenStream2;

use quote::quote;

use crate::struct_gen::StructGenerator;

/// The generator of the implementation of `BuildableValue` for a struct with named fields.
pub struct StructWithUnnamedFields<'s> {
    /// The base struct generator.
    gen: &'s StructGenerator,
}

impl<'s> StructWithUnnamedFields<'s> {
    /// Make a new `StructWithNamedFields` from a `StructGenerator`.
    pub fn new(gen: &'s StructGenerator) -> Self {
        if gen.fields.len() != 1 {
            abort!(
                gen.span,
                "structs with unnamed fields must have only one field"
            );
        }
        Self { gen }
    }

    /// Generate the implementation of the trait methods.
    pub fn gen(&self) -> TokenStream2 {
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
    fn gen_fn_apply(&self) -> TokenStream2 {
        quote! {
            fn apply(&mut self, data: &str, current_fields: &[String]) -> Result<(), ibuilder::ChooseError> {
                self.0.apply(data, current_fields)
            }
        }
    }

    /// Generate the implementation of the `get_options` method.
    fn gen_fn_get_options(&self) -> TokenStream2 {
        quote! {
            fn get_options(&self, current_fields: &[String]) -> ibuilder::Options {
                self.0.get_options(current_fields)
            }
        }
    }

    /// Generate the implementation of the `get_subfields` method.
    fn gen_fn_get_subfields(&self) -> TokenStream2 {
        quote! {
            fn get_subfields(&self, current_fields: &[String]) -> Vec<String> {
                self.0.get_subfields(current_fields)
            }
        }
    }

    /// Generate the implementation of the `to_node` method.
    fn gen_fn_to_node(&self) -> TokenStream2 {
        let name = self.gen.actual_name();
        quote! {
            fn to_node(&self) -> ibuilder::nodes::Node {
                ibuilder::nodes::Node::Composite(
                    #name.to_string(),
                    vec![ibuilder::nodes::FieldKind::Unnamed(self.0.to_node())],
                )
            }
        }
    }

    /// Generate the implementation of the `get_value_any` method.
    fn gen_fn_get_value_any(&self) -> TokenStream2 {
        let ident = &self.gen.ident;
        quote! {
            fn get_value_any(&self) -> Option<Box<dyn std::any::Any>> {
                Some(Box::new(#ident(*self.0.get_value_any()?.downcast().unwrap())))
            }
        }
    }
}
