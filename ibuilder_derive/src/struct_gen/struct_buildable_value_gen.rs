use syn::export::TokenStream2;

use quote::quote;

use crate::struct_gen::named_fields::StructWithNamedFields;
use crate::struct_gen::StructGenerator;

/// Generate the implementation of `BuildableValue` for the builder struct.
pub fn gen_impl_buildable_value(gen: &StructGenerator) -> TokenStream2 {
    let builder_ident = &gen.builder_ident;
    let content = if gen.is_named() {
        StructWithNamedFields::new(gen).gen()
    } else {
        todo!("StructUnnamed")
    };
    quote! {
        #[automatically_derived]
        impl ibuilder::BuildableValue for #builder_ident {
            #content
        }
    }
}
