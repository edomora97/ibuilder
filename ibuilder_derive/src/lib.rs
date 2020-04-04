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

use quote::ToTokens;

use proc_macro_error::{abort, proc_macro_error};

use crate::enum_gen::EnumGenerator;
use crate::struct_gen::StructGenerator;

mod enum_gen;
mod struct_gen;

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
    match &ast.data {
        syn::Data::Struct(_) => StructGenerator::from_struct(ast).to_token_stream().into(),
        syn::Data::Enum(_) => EnumGenerator::from_enum(ast).to_token_stream().into(),
        _ => abort!(ast, "only structs can derive ibuilder"),
    }
}
