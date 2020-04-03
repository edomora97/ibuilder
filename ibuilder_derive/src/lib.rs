//! See the documentation of the [`ibuilder`](https://crates.io/ibuilder) create for the details,
//! you probably are looking for that.
//!
//! ## ibuilder derive macro
//!
//! Usage:
//! ```
//! # use ibuilder_derive::ibuilder;
//! #[derive(ibuilder)]
//! struct Example {
//!     /// The help message for field1
//!     field1: i64,
//!     /// The help message for field2
//!     #[ibuilder(default = "something")]
//!     field2: String,
//! }
//! ```
//!
//! Will implement the function `Example::builder()` that returns a `Builder<Example>` for
//! interactively building instances of the `Example` struct.
//!
//! It will also implement a private struct for keeping the state of the builder and implement the
//! `NewBuildableValue` trait for `Example`.

use proc_macro::TokenStream;

use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};

use crate::buildable_value_impl::{
    gen_impl_buildable_value, gen_impl_struct_buildable_value, gen_struct_buildable_value,
};
use crate::builder_impl::gen_builder_gen_method;

mod buildable_value_impl;
mod builder_impl;
mod field_data;

/// Derive macro for `ibuilder`.
#[proc_macro_error]
#[proc_macro_derive(ibuilder, attributes(ibuilder))]
pub fn ibuilder_derive(input: TokenStream) -> TokenStream {
    ibuilder_macro(&syn::parse(input).unwrap())
}

/// Main macro body. It generates the following constructs:
/// - `impl Name { fn builder() -> Builder<Name> { ... } }`
/// - `struct __Name_BuildableValueImpl` with the `BuildableValue` fields
/// - `impl BuildableValue for __Name_BuildableValueImpl`
/// - `impl NewBuildableValue for Name`
fn ibuilder_macro(ast: &syn::DeriveInput) -> TokenStream {
    // allow the derivation only on structs with named fields
    let data = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(_) => data,
            _ => abort!(ast, "Only structs with named fields are supported"),
        },
        _ => abort!(ast, "only structs can derive ibuilder"),
    };

    // the original name of the derived struct
    let name = &ast.ident;
    // the name of the newly created struct that implements `BuildableValue`.
    let buildable_value_name = format_ident!("__{}_BuildableValueImpl", name);

    let impl_builder_method = gen_builder_gen_method(&buildable_value_name, &name);
    let struct_buildable_value = gen_struct_buildable_value(data, &buildable_value_name);
    let impl_struct_buildable_value = gen_impl_struct_buildable_value(data, &buildable_value_name);
    let impl_buildable_value = gen_impl_buildable_value(data, &name, &buildable_value_name);

    let gen = quote! {
        use ibuilder::NewBuildableValue as _;
        #impl_builder_method
        #struct_buildable_value
        #impl_struct_buildable_value
        #impl_buildable_value
    };
    gen.into()
}
