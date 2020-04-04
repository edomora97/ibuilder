use crate::enum_gen::enum_buildable_value_gen::gen_impl_buildable_value;
use crate::struct_gen::{StructField, StructGenerator};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::export::TokenStream2;
use syn::{Fields, Ident, Variant};

mod enum_buildable_value_gen;

/// Generator for all the builder-related tokens for an `enum`. It will generate some new structs
/// and enums that implements `BuildableValue`, and will implement some traits for the various types
/// in the game.
#[derive(Debug)]
pub struct EnumGenerator {
    /// The `Ident` of the original enum.
    ident: Ident,
    /// The `Ident` of the struct that implements `BuildableValue` for the enum.
    builder_ident: Ident,
    /// The `Ident` of the enum that contains the state of the `BuildableValue`.
    variants_builder_ident: Ident,
    /// The list of the variants of the original enum.
    variants: Vec<EnumVariant>,
}

/// The information about a variant of an enum.
#[derive(Debug)]
pub struct EnumVariant {
    /// The `Ident` of the variant.
    ident: Ident,
    /// The kind of the variant.
    kind: VariantKind,
}

/// The information about the type of variant.
#[derive(Debug)]
pub enum VariantKind {
    /// The variant doesn't contain any field.
    Empty,
    /// The variant contains only unnamed fields.
    Unnamed(Vec<StructField>),
    /// The variant contains only named fields.
    Named(Vec<StructField>),
}

/// Generator for the list of variant definition of an enum.
struct VariantsDefList<'s> {
    /// A reference to the original generator.
    gen: &'s EnumGenerator,
}

impl EnumGenerator {
    /// Construct a new `EnumGenerator` from the AST of an enum. Will fail if the AST is not
    /// relative to an enum.
    pub fn from_enum(ast: &syn::DeriveInput) -> EnumGenerator {
        match &ast.data {
            syn::Data::Enum(data) => EnumGenerator {
                ident: ast.ident.clone(),
                builder_ident: gen_builder_ident(&ast.ident),
                variants_builder_ident: gen_variants_builder_ident(&ast.ident),
                variants: data.variants.iter().map(EnumVariant::from).collect(),
            },
            _ => panic!("expecting an enum"),
        }
    }

    /// Make a new `VariantsDefList` for this enum.
    fn variants_def_list(&self) -> VariantsDefList {
        VariantsDefList { gen: self }
    }
}

impl EnumVariant {
    /// Generate (or not in case of empty variants) a structure that contains the internal state
    /// of a variant. This struct will have the same fields as the variant, and derives from
    /// `ibuilder`.
    fn gen_builder(&self, ident: Ident) -> TokenStream2 {
        let fields_def = match &self.kind {
            VariantKind::Empty => return TokenStream2::new(),
            VariantKind::Unnamed(fields) => {
                let fields: Vec<_> = fields.iter().map(|f| &f.field).collect();
                quote! { (#(#fields,)*); }
            }
            VariantKind::Named(fields) => {
                let fields: Vec<_> = fields.iter().map(|f| &f.field).collect();
                quote! { { #(#fields,)* } }
            }
        };
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug, ibuilder)]
            struct #ident #fields_def
        }
    }

    /// Return the tokens for initializing the builder of this variant.
    fn builder_new(&self, base: &Ident) -> TokenStream2 {
        let variant = &self.ident;
        let builder = gen_variants_builder_ident(base);
        let variant_builder = gen_variants_builder_variant_ident(base, variant);
        let variant_builder = StructGenerator::gen_builder_ident(&variant_builder);
        match &self.kind {
            VariantKind::Empty => quote! { #builder::#variant },
            VariantKind::Unnamed(_) | VariantKind::Named(_) => {
                quote! { #builder::#variant(#variant_builder::new()) }
            }
        }
    }

    /// Return the list with the names of all the named fields in this variant.
    fn field_names(&self) -> Vec<Ident> {
        match &self.kind {
            VariantKind::Unnamed(_) | VariantKind::Empty => vec![],
            VariantKind::Named(fields) => fields.iter().map(|f| f.ident.clone().unwrap()).collect(),
        }
    }
}

impl VariantKind {
    /// Check if this is `VariantKind::Empty`.
    fn is_empty(&self) -> bool {
        match self {
            VariantKind::Empty => true,
            _ => false,
        }
    }
}

impl From<&Variant> for EnumVariant {
    fn from(variant: &Variant) -> EnumVariant {
        EnumVariant {
            ident: variant.ident.clone(),
            kind: match &variant.fields {
                Fields::Named(fields) => {
                    VariantKind::Named(fields.named.iter().map(StructField::from).collect())
                }
                Fields::Unnamed(fields) => {
                    VariantKind::Unnamed(fields.unnamed.iter().map(StructField::from).collect())
                }
                Fields::Unit => VariantKind::Empty,
            },
        }
    }
}

/// Generate the `Ident` to use as the implementation of `BuildableValue` for an enum. The type of
/// this element is `struct`.
fn gen_builder_ident(ident: &Ident) -> Ident {
    format_ident!("__{}_BuildableValueImpl", ident)
}

/// Generate the `Ident` to use as the inner state of the `BuildableValue` of the enum. The type
/// of this element is `enum`.
fn gen_variants_builder_ident(ident: &Ident) -> Ident {
    format_ident!("__{}_Variants_BuildableValueImpl", ident)
}

/// Generate the `Ident` to use as the state of a variant with fields. The type of this element is
/// `struct`.
fn gen_variants_builder_variant_ident(ident: &Ident, variant: &Ident) -> Ident {
    format_ident!("__{}_Variants_{}", ident, variant)
}

impl ToTokens for EnumGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append_all(gen_struct_builder(self));
        tokens.append_all(gen_variants_builder(self));
        // generate the structs for keeping the state of the fields of the variants
        for variant in &self.variants {
            tokens.append_all(variant.gen_builder(gen_variants_builder_variant_ident(
                &self.ident,
                &variant.ident,
            )));
        }
        tokens.append_all(gen_impl_new_buildable_value(self));
        tokens.append_all(gen_impl_buildable_value(self));
    }
}

impl ToTokens for VariantsDefList<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for variant in &self.gen.variants {
            let ident = &variant.ident;
            tokens.append_all(quote! {#ident});
            match &variant.kind {
                VariantKind::Unnamed(_) | VariantKind::Named(_) => {
                    let variant_builder =
                        gen_variants_builder_variant_ident(&self.gen.ident, ident);
                    let variant_builder = StructGenerator::gen_builder_ident(&variant_builder);
                    tokens.append_all(quote! { (#variant_builder) });
                }
                VariantKind::Empty => {}
            }
            tokens.append_all(quote! {,});
        }
    }
}

/// Generate the structure that implements `BuildableValue` for the enum, and implement the `new()`
/// function for it.
fn gen_struct_builder(gen: &EnumGenerator) -> TokenStream2 {
    let builder_ident = &gen.builder_ident;
    let variants_builder_ident = &gen.variants_builder_ident;
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(Debug)]
        struct #builder_ident {
            value: Option<#variants_builder_ident>,
        }

        #[automatically_derived]
        impl #builder_ident {
            fn new() -> #builder_ident {
                #builder_ident { value: None }
            }
        }
    }
}

/// Generate the enum that contains the internal state of the `BuildableValue` for the enum.
fn gen_variants_builder(gen: &EnumGenerator) -> TokenStream2 {
    let variants_builder_ident = &gen.variants_builder_ident;
    let variants = gen.variants_def_list();
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(Debug)]
        enum #variants_builder_ident {
            #variants
        }
    }
}

/// Generate the implementation of `NewBuildableValue` for the enum.
fn gen_impl_new_buildable_value(gen: &EnumGenerator) -> TokenStream2 {
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
