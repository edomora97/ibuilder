//! [![crates.io](https://img.shields.io/crates/v/ibuilder_derive.svg)](https://crates.io/crates/ibuilder_derive)
//! [![Docs](https://docs.rs/ibuilder_derive/badge.svg)](https://docs.rs/ibuilder_derive)
//!
//! See the documentation of the [`ibuilder`](https://crates.io/crates/ibuilder) create for the details,
//! you probably are looking for that.
//!
//! ## ibuilder derive macro
//!
//! Usage:
//! ```
//! # use ibuilder_derive::IBuilder;
//! #[derive(IBuilder)]
//! struct Example {
//!     /// The help message for field1
//!     field1: i64,
//!     /// The help message for field2
//!     #[ibuilder(default = "something")]
//!     field2: String,
//! }
//! ```
//!
//! Will implement the trait `ibuilder::Buildable` for `Example`, prodiding the `builder()` method
//! for getting a `ibuilder::Builder`.
//!
//! It will also implement a private struct for keeping the state of the builder and implement the
//! `NewBuildableValue` trait for `Example`, allowing it to be inside a fields of other derived
//! types.

use proc_macro::TokenStream;

use quote::{quote, ToTokens};

use proc_macro_error::{abort, proc_macro_error, set_dummy};

use crate::enum_gen::EnumGenerator;
use crate::struct_gen::StructGenerator;

mod enum_gen;
mod struct_gen;

/// Derive macro for `IBuilder`.
#[proc_macro_error]
#[proc_macro_derive(IBuilder, attributes(ibuilder))]
pub fn ibuilder_derive(input: TokenStream) -> TokenStream {
    ibuilder_macro(&syn::parse(input).unwrap())
}

/// Main macro body. It generates the following constructs:
/// - `impl Buildable for Name`
/// - `impl NewBuildableValue for Name`
/// - Some private structure and enums for keeping the state of the builder for `Name`, all those
///   types have the name starting with `__`.
fn ibuilder_macro(ast: &syn::DeriveInput) -> TokenStream {
    fix_double_error(&ast.ident);
    match &ast.data {
        syn::Data::Struct(_) => StructGenerator::from_struct(ast).to_token_stream().into(),
        syn::Data::Enum(_) => EnumGenerator::from_enum(ast).to_token_stream().into(),
        _ => abort!(ast, "only structs can derive ibuilder"),
    }
}

/// Parse the string attribute into the `Option<String>`. In case of duplicate or not string a
/// compile error is raised.
fn parse_string_meta(out: &mut Option<String>, lit: syn::Lit) {
    if out.is_none() {
        match lit {
            syn::Lit::Str(content) => *out = Some(content.value()),
            _ => abort!(lit, "expecting a string"),
        }
    } else {
        abort!(lit, "duplicated attribute");
    }
}

fn fix_double_error(ident: &syn::Ident) {
    set_dummy(quote! {
        impl ibuilder::Buildable<#ident> for #ident {
            fn builder() -> ibuilder::Builder<#ident> { unimplemented!() }
        }
    });
}
