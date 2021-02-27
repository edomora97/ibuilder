use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_warning, ResultExt};
use syn::punctuated::Punctuated;
use syn::{Fields, Ident, Meta, MetaNameValue, Token, Variant};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};

use crate::enum_gen::enum_buildable_value_gen::gen_impl_buildable_value;
use crate::parse_string_meta;
use crate::struct_gen::{StructField, StructGenerator};

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
    /// The metadata associated with the enum.
    metadata: EnumMetadata,
}

/// The metadata of the enum, it's taken from the attributes of the `enum`.
#[derive(Debug)]
pub struct EnumMetadata {
    /// The prompt to use for this enum's main menu.
    prompt: Option<String>,
}

/// The information about a variant of an enum.
#[derive(Debug)]
pub struct EnumVariant {
    /// The `Ident` of the variant.
    ident: Ident,
    /// The kind of the variant.
    kind: VariantKind,
    /// The metadata associated with the variant.
    metadata: VariantMetadata,
}

/// The metadata of the variant, it's taken from the attributes of the `Variant`.
#[derive(Debug)]
pub struct VariantMetadata {
    /// The prompt to use for this variant.
    prompt: Option<String>,
    /// Different name to use in the tree structure.
    rename: Option<String>,
    /// Whether this variant is hidden.
    hidden: bool,
    /// Whether this is the default variant.
    default: bool,
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
            syn::Data::Enum(data) => {
                let generator = EnumGenerator {
                    ident: ast.ident.clone(),
                    builder_ident: gen_builder_ident(&ast.ident),
                    variants_builder_ident: gen_variants_builder_ident(&ast.ident),
                    variants: data.variants.iter().map(EnumVariant::from).collect(),
                    metadata: EnumMetadata::from(ast),
                };
                if generator.variants.iter().all(|v| v.metadata.hidden) {
                    abort!(ast, "all the variants are hidden");
                }
                if generator
                    .variants
                    .iter()
                    .filter(|v| v.metadata.default)
                    .count()
                    > 1
                {
                    abort!(ast, "at most one variant can be the default");
                }
                generator
            }
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
    /// `IBuilder`.
    fn gen_builder(&self, ident: Ident) -> TokenStream {
        let name = self.actual_name();
        let mut attrs = Vec::new();
        if let Some(prompt) = &self.metadata.prompt {
            attrs.push(quote! { prompt = #prompt });
        }
        attrs.push(quote! { rename = #name });
        let fields_def = match &self.kind {
            VariantKind::Empty => return TokenStream::new(),
            VariantKind::Unnamed(fields) => {
                let fields: Vec<_> = fields.iter().map(|f| &f.field).collect();
                if fields.len() != 1 {
                    abort!(
                        self.ident,
                        "variants with unnamed fields are supported only with one field"
                    );
                }
                quote! { (#(#fields,)*); }
            }
            VariantKind::Named(fields) => {
                let fields: Vec<_> = fields.iter().map(|f| &f.field).collect();
                quote! { { #(#fields,)* } }
            }
        };
        quote! {
            #[allow(non_camel_case_types)]
            #[derive(IBuilder)]
            #[ibuilder(#(#attrs,)*)]
            struct #ident #fields_def
        }
    }

    /// Return the tokens for initializing the builder of this variant.
    fn builder_new(&self, base: &Ident) -> TokenStream {
        let variant = &self.ident;
        let builder = gen_variants_builder_ident(base);
        let variant_builder = gen_variants_builder_variant_ident(base, variant);
        let variant_builder = StructGenerator::gen_builder_ident(&variant_builder);
        match &self.kind {
            VariantKind::Empty => quote! { #builder::#variant },
            VariantKind::Unnamed(_) | VariantKind::Named(_) => {
                let prompt = match &self.metadata.prompt {
                    Some(prompt) => quote! {Some(#prompt.into())},
                    None => quote! {None},
                };
                quote! {
                    #builder::#variant(#variant_builder::new(ibuilder::BuildableValueConfig {
                        default: None,
                        prompt: #prompt,
                    }))
                }
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

    /// Return the actual name of the variant, which is the defined name or the renamed one. The
    /// string literal of the name is returned.
    fn actual_name(&self) -> TokenStream {
        if let Some(renamed) = &self.metadata.rename {
            quote! { #renamed }
        } else {
            let ident = self.ident.to_string();
            quote! { #ident }
        }
    }
}

impl From<&syn::DeriveInput> for EnumMetadata {
    fn from(data: &syn::DeriveInput) -> EnumMetadata {
        let mut metadata = EnumMetadata { prompt: None };
        for attr in &data.attrs {
            if attr.path.is_ident("ibuilder") {
                let meta = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .unwrap_or_abort();
                for meta in meta {
                    parse_enum_meta(meta, &mut metadata);
                }
            }
        }
        metadata
    }
}

/// Extract the `EnumMetadata` from a `Meta` entry in an attribute. `meta` comes from
/// `#[ibuilder(HERE)]`.
fn parse_enum_meta(meta: Meta, metadata: &mut EnumMetadata) {
    match meta {
        Meta::NameValue(MetaNameValue { path, lit, .. }) => {
            if path.is_ident("prompt") {
                parse_string_meta(&mut metadata.prompt, lit);
            } else if path.is_ident("rename") {
                abort!(
                    path,
                    "renaming an enum is not supported since the name is not exposed"
                );
            } else {
                abort!(path, "unknown attribute");
            }
        }
        _ => abort!(meta, "unknown attribute"),
    }
}

impl VariantKind {
    /// Check if this is `VariantKind::Empty`.
    fn is_empty(&self) -> bool {
        matches!(self, VariantKind::Empty)
    }
}

impl From<&Variant> for EnumVariant {
    fn from(variant: &Variant) -> EnumVariant {
        let metadata = VariantMetadata::from(variant);
        EnumVariant {
            ident: variant.ident.clone(),
            kind: match &variant.fields {
                Fields::Named(fields) => {
                    VariantKind::Named(fields.named.iter().map(StructField::from).collect())
                }
                Fields::Unnamed(fields) => {
                    let mut fields: Vec<_> = fields.unnamed.iter().map(StructField::from).collect();
                    // forward the prompt to the unnamed fields to avoid having to add the attribute
                    // for the field (i.e. inside the parenthesis).
                    if let Some(prompt) = &metadata.prompt {
                        for field in fields.iter_mut() {
                            if field.metadata.prompt.is_none() {
                                field.metadata.prompt = Some(prompt.clone());
                            }
                        }
                    }
                    VariantKind::Unnamed(fields)
                }
                Fields::Unit => {
                    if metadata.prompt.is_some() {
                        abort!(variant, "prompt not supported for empty variants");
                    }
                    VariantKind::Empty
                }
            },
            metadata,
        }
    }
}

impl From<&Variant> for VariantMetadata {
    fn from(var: &Variant) -> Self {
        let mut metadata = VariantMetadata {
            prompt: None,
            rename: None,
            hidden: false,
            default: false,
        };
        for attr in &var.attrs {
            if attr.path.is_ident("ibuilder") {
                let meta = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .unwrap_or_abort();
                for meta in meta {
                    parse_variant_meta(meta, &mut metadata);
                }
            }
        }
        metadata
    }
}

/// Extract the `VariantMetadata` from a `Meta` entry in a variant attribute. `meta` comes from
/// `#[ibuilder(HERE)]`.
fn parse_variant_meta(meta: Meta, metadata: &mut VariantMetadata) {
    match meta {
        Meta::NameValue(MetaNameValue { path, lit, .. }) => {
            if path.is_ident("prompt") {
                parse_string_meta(&mut metadata.prompt, lit);
            } else if path.is_ident("rename") {
                parse_string_meta(&mut metadata.rename, lit);
            } else {
                abort!(path, "unknown attribute");
            }
        }
        Meta::Path(path) => {
            if path.is_ident("hidden") {
                if metadata.hidden {
                    emit_warning!(path, "duplicated attribute");
                }
                metadata.hidden = true;
            } else if path.is_ident("default") {
                if metadata.default {
                    emit_warning!(path, "duplicated attribute");
                }
                metadata.default = true;
            } else {
                abort!(path, "unknown attribute");
            }
        }
        _ => abort!(meta, "unknown attribute"),
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
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
fn gen_struct_builder(gen: &EnumGenerator) -> TokenStream {
    let builder_ident = &gen.builder_ident;
    let variants_builder_ident = &gen.variants_builder_ident;
    let prompt = if let Some(prompt) = &gen.metadata.prompt {
        prompt
    } else {
        "Select a variant"
    };
    let mut default = quote! { None };
    for var in &gen.variants {
        if var.metadata.default {
            let init = var.builder_new(&gen.ident);
            default = quote! { Some(#init) };
        }
    }
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #[derive(Debug)]
        struct #builder_ident {
            value: Option<#variants_builder_ident>,
            prompt: String,
        }

        #[automatically_derived]
        impl #builder_ident {
            fn new(config: ibuilder::BuildableValueConfig<()>) -> #builder_ident {
                #builder_ident {
                    value: #default,
                    prompt: config.prompt.unwrap_or_else(|| #prompt.to_string())
                }
            }
        }
    }
}

/// Generate the enum that contains the internal state of the `BuildableValue` for the enum.
fn gen_variants_builder(gen: &EnumGenerator) -> TokenStream {
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
fn gen_impl_new_buildable_value(gen: &EnumGenerator) -> TokenStream {
    let ident = &gen.ident;
    let builder_ident = &gen.builder_ident;
    quote! {
        #[automatically_derived]
        impl ibuilder::NewBuildableValue for #ident {
            fn new_buildable_value(config: ibuilder::BuildableValueConfig<()>) -> Box<dyn ibuilder::BuildableValue> {
                Box::new(#builder_ident::new(config))
            }
        }
    }
}
