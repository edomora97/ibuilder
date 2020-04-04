use proc_macro_error::{abort, ResultExt};
use syn::export::TokenStream2;
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Ident, Meta, MetaNameValue, Token, Type};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};

use crate::struct_gen::struct_buildable_value_gen::gen_impl_buildable_value;

mod named_fields;
mod struct_buildable_value_gen;

/// Generator for all the builder-related tokens for a `struct`. It will generate a new struct that
/// implements `BuildableValue`, and will implement some traits for the various types in the game.
#[derive(Debug)]
pub struct StructGenerator {
    /// The `Ident` of the original struct.
    ident: Ident,
    /// The `Ident` of the newly created struct.
    builder_ident: Ident,
    /// The list of fields in the original struct.
    fields: Vec<StructField>,
}

/// The information about a field of a struct.
#[derive(Debug)]
pub struct StructField {
    /// The `Ident` of the field. It's `None` if the struct has unnamed fields (`struct Foo(i64)`).
    pub ident: Option<Ident>,
    /// The type of the field.
    pub ty: Type,
    /// The whole field that generated this instance.
    pub field: Field,
    /// The metadata associated with the field.
    pub metadata: FieldMetadata,
}

/// The metadata of the field, it's taken from the attributes of the `Field`.
#[derive(Debug)]
pub struct FieldMetadata {
    /// The default value for this field. Only available if the field type is a builtin type. It is
    /// None if the default value is not specified, otherwise it's
    /// `Some( quote!{ Some(value.into()) })`.
    default: Option<TokenStream2>,
}

/// Generator for the list of field definition of a struct. It will generate either:
/// - `{ name: Type, name: Type, ... }`
/// - `(Type, Type, ...)`
///
/// Where `Type` is the type of the generator that is able to build the corresponding field of the
/// original struct. For builtin types it's `ibuilder::builders::XXXBuilder`, for the custom types
/// it's `Box<dyn BuildableValue>`.
struct FieldDefList<'s> {
    /// A reference to the list of fields of the struct.
    fields: &'s [StructField],
    /// Whether the struct has named fields or not.
    named: bool,
}

/// Generator for the list of field creation of a struct. It will generate either:
/// - `{ name: Type::new(...), ... }`
/// - `(Type::new(...), ...)`
///
/// Where the `Type::new` will be chosen according to the type of the field in the original struct
/// and eventually will forward the `FieldMetadata`.
struct FieldNewList<'s> {
    /// A reference to the original generator for the struct.
    gen: &'s StructGenerator,
}

impl StructGenerator {
    /// Generate the `Ident` to use as the implementation of `BuildableValue` for a struct.
    pub fn gen_builder_ident(ident: &Ident) -> Ident {
        format_ident!("__{}_BuildableValueImpl", ident)
    }

    /// Construct a new `StructGenerator` from the AST of a struct. Will fail if the AST is not
    /// relative to a struct.
    pub fn from_struct(ast: &syn::DeriveInput) -> StructGenerator {
        match &ast.data {
            syn::Data::Struct(data) => StructGenerator {
                ident: ast.ident.clone(),
                builder_ident: StructGenerator::gen_builder_ident(&ast.ident),
                fields: match &data.fields {
                    syn::Fields::Named(fields) => {
                        fields.named.iter().map(StructField::from).collect()
                    }
                    syn::Fields::Unnamed(fields) => {
                        fields.unnamed.iter().map(StructField::from).collect()
                    }
                    syn::Fields::Unit => vec![],
                },
            },
            _ => panic!("expecting a struct"),
        }
    }

    /// Whether the fields of the original struct are named or unnamed (`struct Foo(i64)`).
    fn is_named(&self) -> bool {
        self.fields.iter().all(|f| f.ident.is_some())
    }

    /// The list of the `Ident` of the named fields in the original struct.
    fn field_names(&self) -> Vec<Ident> {
        self.fields.iter().filter_map(|f| f.ident.clone()).collect()
    }

    /// Make a new `FieldDefList` relative to this struct.
    fn fields_def_list(&self) -> FieldDefList {
        FieldDefList {
            fields: &self.fields,
            named: self.is_named(),
        }
    }

    /// Make a new `FieldNewList` relative to this struct.
    fn fields_new_list(&self) -> FieldNewList {
        FieldNewList { gen: &self }
    }
}

impl StructField {
    /// The type of the builder for the type of this field. It's either one of the builtin types or
    /// a generic boxed one.
    fn builder_type(&self) -> TokenStream2 {
        if let Some(builtin) = self.builtin_type() {
            quote! { #builtin }
        } else {
            quote! { Box<dyn ibuilder::BuildableValue> }
        }
    }

    /// The initializer of the builder for the current field. It will forward the `FieldMetadata`
    /// to the builder.
    fn builder_new(&self) -> TokenStream2 {
        if let Some(builtin) = self.builtin_type() {
            let default = self
                .metadata
                .default
                .clone()
                .unwrap_or_else(|| quote! {None});
            quote! { <#builtin>::new(#default) }
        } else {
            let ty = &self.ty;
            quote! { <#ty as ibuilder::NewBuildableValue>::new_buildable_value() }
        }
    }

    /// Check if the type of the field is a builtin type, and in this case it will return the
    /// corresponding builder. It returns `None` if it's not a builtin type.
    fn builtin_type(&self) -> Option<TokenStream2> {
        match &self.ty {
            Type::Path(path) => {
                let segments = &path.path.segments;
                if segments.len() != 1 {
                    return None;
                }
                let ty = segments[0].ident.to_string();
                let ty = ty.as_str();
                match ty {
                    "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize"
                    | "usize" | "f32" | "f64" | "String" | "char" | "bool" => {
                        let builder =
                            format_ident!("{}", ty[0..1].to_uppercase() + &ty[1..] + "Builder");
                        Some(quote! { ibuilder::builders::#builder })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl ToTokens for StructGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append_all(gen_struct_builder(&self));
        tokens.append_all(gen_impl_new_buildable_value(&self));
        tokens.append_all(gen_impl_buildable_value(&self));
    }
}

impl<'s> ToTokens for FieldDefList<'s> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // an empty struct is declared like `struct Foo;`
        if self.fields.is_empty() {
            tokens.append_all(quote! { ; });
            return;
        }
        let mut inner = TokenStream2::new();
        for field in self.fields {
            // named field: prepend the field name
            if let Some(ident) = &field.ident {
                inner.append_all(quote! {#ident: });
            }
            let ty = field.builder_type();
            inner.append_all(quote! {#ty,})
        }
        if self.named {
            tokens.append_all(quote! { { #inner } });
        } else {
            tokens.append_all(quote! { ( #inner ); });
        }
    }
}

impl<'s> ToTokens for FieldNewList<'s> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.gen.fields.is_empty() {
            tokens.append_all(quote! {});
            return;
        }
        let mut inner = TokenStream2::new();
        for field in &self.gen.fields {
            // named field: prepend the field name
            if let Some(ident) = &field.ident {
                inner.append_all(quote! {#ident: });
            }
            let init = field.builder_new();
            inner.append_all(quote! {#init,})
        }
        if self.gen.is_named() {
            tokens.append_all(quote! { { #inner } });
        } else {
            tokens.append_all(quote! { ( #inner ) });
        }
    }
}

impl From<&Field> for StructField {
    fn from(field: &Field) -> StructField {
        let res = StructField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            field: field.clone(),
            metadata: get_field_metadata(&field.attrs),
        };
        if res.metadata.default.is_some() && res.builtin_type().is_none() {
            abort!(field, "default value is supported only on plain types");
        }
        res
    }
}

/// Extract the `FieldMetadata` from the attribute list of a field.
fn get_field_metadata(attrs: &[Attribute]) -> FieldMetadata {
    let mut metadata = FieldMetadata { default: None };
    for attr in attrs {
        if attr.path.is_ident("ibuilder") {
            let meta = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .unwrap_or_abort();
            for meta in meta {
                parse_field_meta(meta, &mut metadata);
            }
        }
    }
    metadata
}

/// Extract the `FieldMetadata` from a `Meta` entry in a field attribute. `meta` comes from
/// `#[ibuilder(HERE)]`.
fn parse_field_meta(meta: Meta, metadata: &mut FieldMetadata) {
    match meta {
        Meta::NameValue(MetaNameValue { path, lit, .. }) => {
            if path.is_ident("default") {
                if metadata.default.is_none() {
                    metadata.default = Some(quote! {Some(#lit.into())});
                } else {
                    abort!(path, "duplicated default");
                }
            } else {
                abort!(path, "unknown attribute");
            }
        }
        _ => abort!(meta, "unknown attribute"),
    }
}

/// Generate the struct that implements `BuildableValue` for the struct, and implement the `new()`
/// function for it.
fn gen_struct_builder(gen: &StructGenerator) -> TokenStream2 {
    let builder_ident = &gen.builder_ident;
    let fields_gen = gen.fields_def_list();
    let fields_new = gen.fields_new_list();
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(Debug)]
        struct #builder_ident #fields_gen

        #[automatically_derived]
        impl #builder_ident {
            fn new() -> #builder_ident {
                #builder_ident #fields_new
            }
        }
    }
}

/// Generate the implementation of `NewBuildableValue` for the struct.
fn gen_impl_new_buildable_value(gen: &StructGenerator) -> TokenStream2 {
    let ident = &gen.ident;
    let builder_ident = &gen.builder_ident;
    quote! {
        #[automatically_derived]
        impl ibuilder::NewBuildableValue for #ident {
            fn new_buildable_value() -> Box<dyn ibuilder::BuildableValue> {
                Box::new(#builder_ident::new())
            }
        }
    }
}
