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
//!     field1: i64,
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
///
/// # Supported features
/// This is a somewhat complete list of all the attributes it's possible to use deriving from
/// `IBuilder`.
///
/// ## `#[ibuilder(rename = "new name")]`
/// When applied to a struct, a named field (of a struct or of a variant) or an enum's variant it
/// changes the named displayed in the list of possible options returned by `get_options()` and in
/// the tree structure returned by `to_node`.
///
/// Renaming an enum is not is supported since that name is not shown anywhere.
///
/// ```
/// # use ibuilder_derive::IBuilder;
/// #[derive(IBuilder)]
/// #[ibuilder(rename = "new struct name")]
/// struct Struct {
///     #[ibuilder(rename = "new field name")]
///     field1: i64,
/// }
/// #[derive(IBuilder)]
/// enum Enum {
///     #[ibuilder(rename = "new variant name")]
///     Var1,
///     Var2 {
///         #[ibuilder(rename = "new field name")]
///         field: i32,
///     },
/// }
/// ```
///
/// ## `#[ibuilder(prompt = "new prompt message")]`
/// Change the message attached to the result of `get_options()` for a struct, an enum, a field or a
/// variant. The prompt set on fields and variants overwrites the one on the structs and enum. If
/// not specified a default value is used.
///
/// ```
/// # use ibuilder_derive::IBuilder;
/// #[derive(IBuilder)]
/// #[ibuilder(prompt = "new struct prompt")]
/// struct Struct {
///     #[ibuilder(rename = "new field prompt")]
///     field1: i64,
/// }
/// #[derive(IBuilder)]
/// #[ibuilder(prompt = "new enum prompt")]
/// enum Enum {
///     #[ibuilder(rename = "new variant promp")]
///     Var1,
///     Var2 {
///         #[ibuilder(rename = "new field prompt")]
///         field: i32,
///     },
/// }
/// ```
///
/// ## `#[ibuilder(default = something)]`
/// Set a default value for the field. After the equal sign a literal is expected, if it is a string
/// literal the conversion is done using `FromStr` **at runtime**, otherwise the literal is
/// converted using the `as` syntax.
///
/// For now only the builtin types can be defaulted (numeric types, bool, char and String).
///
/// ```
/// # use ibuilder_derive::IBuilder;
/// #[derive(IBuilder)]
/// struct Struct {
///     #[ibuilder(default = 42)]
///     field1: u8,
///     #[ibuilder(default = "something")]
///     field2: String,
///     #[ibuilder(default = true)]
///     field3: bool,
///     #[ibuilder(default = 'x')]
///     field4: char,
/// }
/// #[derive(IBuilder)]
/// enum Enum {
///     Var {
///         #[ibuilder(default = 42)]
///         field: i32,
///     },
/// }
/// ```
///
/// ## `#[ibuilder(default)]`
/// Set a variant of an enum as the default one for that enum. At most one variant can be set as
/// default.
///
/// ```
/// # use ibuilder_derive::IBuilder;
/// #[derive(IBuilder)]
/// enum Enum {
///     #[ibuilder(default)]
///     Var1,
///     Var2,
/// }
/// ```
///
/// ## `#[ibuilder(hidden)]`
/// Hide a field or a variant from the return value of `get_options()` and `to_node()`. The field
/// cannot be accessed neither using `apply`. If a field is hidden it must have a default value.
///
/// When hiding the fields of an enum, at least one of them must be visible.
///
/// ```
/// # use ibuilder_derive::IBuilder;
/// #[derive(IBuilder)]
/// struct Struct {
///     #[ibuilder(hidden, default = 42)]
///     field1: u8,
/// }
/// #[derive(IBuilder)]
/// enum Enum {
///     Var1,
///     #[ibuilder(hidden)]
///     Var2,
/// }
/// ```
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
