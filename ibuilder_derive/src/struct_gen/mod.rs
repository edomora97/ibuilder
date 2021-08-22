use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, emit_warning, ResultExt};
use syn::punctuated::Punctuated;
use syn::{Field, Fields, Ident, Meta, MetaNameValue, Token, Type};

use quote::{format_ident, quote, ToTokens, TokenStreamExt};

use crate::parse_string_meta;
use crate::struct_gen::struct_buildable_value_gen::gen_impl_buildable_value;

mod named_fields;
mod struct_buildable_value_gen;
mod unnamed_fields;

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
    /// The span of this structure.
    span: Span,
    /// Whether the fields of this struct are named.
    named_fields: bool,
    /// The metadata associated with the struct.
    metadata: StructMetadata,
}

/// The metadata of the struct, it's taken from the attributes of the `struct`.
#[derive(Debug)]
pub struct StructMetadata {
    /// The prompt to use for this struct's main menu.
    prompt: Option<String>,
    /// Different name to use in the tree structure.
    rename: Option<String>,
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
    /// The default value for this field.
    pub default: Option<TokenStream>,
    /// The prompt to use for this field.
    pub prompt: Option<String>,
    /// Different name to use in the tree structure.
    pub rename: Option<String>,
    /// Whether this field is hidden.
    pub hidden: bool,
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

/// Generator for the `impl std::fmt::Debug for ...` implementation block. This will generate a
/// Debug implementation for all the fields, but the hidden ones.
struct ImplDebug<'s> {
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
            syn::Data::Struct(data) => {
                let named_fields = matches!(data.fields, Fields::Named(_));
                let metadata = StructMetadata::from(ast);
                StructGenerator {
                    ident: ast.ident.clone(),
                    builder_ident: StructGenerator::gen_builder_ident(&ast.ident),
                    fields: match &data.fields {
                        syn::Fields::Named(fields) => {
                            fields.named.iter().map(StructField::from).collect()
                        }
                        syn::Fields::Unnamed(fields) => {
                            let mut fields: Vec<_> =
                                fields.unnamed.iter().map(StructField::from).collect();
                            // forward the prompt to the unnamed fields to avoid having to add the
                            // attribute for the field (i.e. inside the parenthesis).
                            if let Some(prompt) = &metadata.prompt {
                                for field in fields.iter_mut() {
                                    if field.metadata.prompt.is_none() {
                                        field.metadata.prompt = Some(prompt.clone());
                                    }
                                }
                            }
                            fields
                        }
                        syn::Fields::Unit => vec![],
                    },
                    span: ast.ident.span(),
                    named_fields,
                    metadata,
                }
            }
            _ => panic!("expecting a struct"),
        }
    }

    /// Return the actual name of the struct, which is the defined name or the renamed one. The
    /// string literal of the name is returned.
    fn actual_name(&self) -> TokenStream {
        if let Some(renamed) = &self.metadata.rename {
            quote! { #renamed }
        } else {
            let ident = self.ident.to_string();
            quote! { #ident }
        }
    }

    /// Whether the fields of the original struct are named or unnamed (`struct Foo(i64)`).
    fn is_named(&self) -> bool {
        self.named_fields
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
        FieldNewList { gen: self }
    }

    /// Make a new `ImplDebug` for to this struct.
    ///
    /// This implements the `Debug` trait without requiring any field to be `Debug`. The basic field
    /// must be `Debug`, but the hidden ones don't have to.
    fn impl_debug(&self) -> ImplDebug {
        ImplDebug { gen: self }
    }
}

impl From<&syn::DeriveInput> for StructMetadata {
    fn from(data: &syn::DeriveInput) -> StructMetadata {
        let mut metadata = StructMetadata {
            prompt: None,
            rename: None,
        };
        for attr in &data.attrs {
            if attr.path.is_ident("ibuilder") {
                let meta = attr
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .unwrap_or_abort();
                for meta in meta {
                    parse_struct_meta(meta, &mut metadata);
                }
            }
        }
        metadata
    }
}

/// Extract the `StructMetadata` from a `Meta` entry in an attribute. `meta` comes from
/// `#[ibuilder(HERE)]`.
fn parse_struct_meta(meta: Meta, metadata: &mut StructMetadata) {
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
        _ => abort!(meta, "unknown attribute"),
    }
}

impl StructField {
    /// The type of the builder for the type of this field. It's either one of the builtin types, a
    /// generic boxed one, or the actual type if the field is hidden.
    fn builder_type(&self) -> TokenStream {
        if self.metadata.hidden {
            let ty = &self.ty;
            quote! { #ty }
        } else if let Some(builtin) = self.builtin_type() {
            quote! { #builtin }
        } else {
            quote! { Box<dyn ibuilder::BuildableValue> }
        }
    }

    /// The initializer of the builder for the current field. It will forward the `FieldMetadata`
    /// to the builder.
    fn builder_new(&self) -> TokenStream {
        let prompt = match &self.metadata.prompt {
            Some(prompt) => quote!(Some(#prompt.to_string())),
            None => quote! {None},
        };
        if self.metadata.hidden {
            return if let Some(default) = &self.metadata.default {
                quote! { #default }
            } else {
                quote! { ::std::default::Default::default() }
            };
        }
        if let Some(builtin) = self.builtin_type() {
            let default = if let Some(default) = self.metadata.default.clone() {
                quote! { Some(#default) }
            } else {
                quote! { None }
            };
            quote! {
                <#builtin>::new(ibuilder::BuildableValueConfig {
                    default: #default,
                    prompt: #prompt,
                })
            }
        } else {
            let ty = &self.ty;
            quote! {
                <#ty as ibuilder::NewBuildableValue>::new_buildable_value(ibuilder::BuildableValueConfig {
                    default: None,
                    prompt: #prompt,
                })
            }
        }
    }

    /// Check if the type of the field is a builtin type, and in this case it will return the
    /// corresponding builder. It returns `None` if it's not a builtin type.
    fn builtin_type(&self) -> Option<TokenStream> {
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

    /// Return the actual name of the field, which is the defined name or the renamed one. The
    /// string literal of the name is returned.
    fn actual_name(&self) -> TokenStream {
        if let Some(renamed) = &self.metadata.rename {
            quote! { #renamed }
        } else {
            let ident = self.ident.as_ref().unwrap().to_string();
            quote! { #ident }
        }
    }
}

impl ToTokens for StructGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(gen_struct_builder(self));
        tokens.append_all(gen_impl_new_buildable_value(self));
        tokens.append_all(gen_impl_buildable_value(self));
    }
}

impl<'s> ToTokens for FieldDefList<'s> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // an empty struct is declared like `struct Foo;`
        if self.fields.is_empty() {
            tokens.append_all(quote! { ; });
            return;
        }
        let mut inner = TokenStream::new();
        for field in self.fields {
            // named field: prepend the field name
            if let Some(ident) = &field.ident {
                inner.append_all(quote! {#ident: });
            }
            let ty = field.builder_type();
            inner.append_all(quote! {#ty,})
        }
        if self.named {
            inner.append_all(quote! { __prompt: String, });
            tokens.append_all(quote! { { #inner } });
        } else {
            // unnamed struct has the prompt directly forwarded to the inner type
            tokens.append_all(quote! { ( #inner ); });
        }
    }
}

impl<'s> ToTokens for FieldNewList<'s> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.gen.fields.is_empty() {
            tokens.append_all(quote! {});
            return;
        }
        let prompt = &self.gen.metadata.prompt.as_deref();
        let prompt = prompt.unwrap_or("Select the field to edit");
        let prompt = quote! { config.prompt.unwrap_or_else(|| #prompt.to_string()) };
        let mut inner = TokenStream::new();
        for field in &self.gen.fields {
            // named field: prepend the field name
            if let Some(ident) = &field.ident {
                inner.append_all(quote! {#ident: });
            }
            let init = field.builder_new();
            inner.append_all(quote! {#init,})
        }
        if self.gen.is_named() {
            inner.append_all(quote! { __prompt: #prompt, });
            tokens.append_all(quote! { { #inner } });
        } else {
            tokens.append_all(quote! { ( #inner ) });
        }
    }
}

impl<'s> ToTokens for ImplDebug<'s> {
    #[allow(clippy::collapsible_else_if)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder_ident = &self.gen.builder_ident;
        let mut fields = TokenStream::new();
        for (i, field) in self.gen.fields.iter().enumerate() {
            if let Some(ident) = &field.ident {
                if field.metadata.hidden {
                    fields.append_all(quote! { .field(stringify!(#ident), &"[hidden]") });
                } else {
                    fields.append_all(quote! { .field(stringify!(#ident), &self.#ident) });
                }
            } else {
                if field.metadata.hidden {
                    fields.append_all(quote! { .field(stringify!(#i), &"[hidden]") });
                } else {
                    let index = syn::Index::from(i);
                    fields.append_all(quote! { .field(stringify!(#i), &self.#index) });
                }
            }
        }
        tokens.append_all(quote! {
            #[automatically_derived]
            impl std::fmt::Debug for #builder_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!(#builder_ident))
                        #fields
                        .finish()
                }
            }
        })
    }
}

impl From<&Field> for StructField {
    fn from(field: &Field) -> StructField {
        let res = StructField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            field: field.clone(),
            metadata: get_field_metadata(field),
        };
        if res.metadata.default.is_some() && res.builtin_type().is_none() {
            abort!(field, "default value is supported only on plain types");
        }
        res
    }
}

/// Extract the `FieldMetadata` from the attribute list of a field.
fn get_field_metadata(field: &Field) -> FieldMetadata {
    let mut metadata = FieldMetadata {
        default: None,
        prompt: None,
        rename: None,
        hidden: false,
    };
    for attr in &field.attrs {
        if attr.path.is_ident("ibuilder") {
            let meta = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .unwrap_or_abort();
            for meta in meta {
                parse_field_meta(meta, &mut metadata, &field.ty);
            }
        }
    }
    if metadata.hidden && field.ident.is_none() {
        abort!(field, "unnamed fields cannot be hidden");
    }
    metadata
}

/// Extract the `FieldMetadata` from a `Meta` entry in a field attribute. `meta` comes from
/// `#[ibuilder(HERE)]`.
fn parse_field_meta(meta: Meta, metadata: &mut FieldMetadata, ty: &Type) {
    match meta {
        Meta::NameValue(MetaNameValue { path, lit, .. }) => {
            if path.is_ident("default") {
                if metadata.default.is_none() {
                    match lit {
                        syn::Lit::Str(_) => {
                            metadata.default =
                                Some(quote! { <#ty as std::str::FromStr>::from_str(#lit).unwrap() })
                        }
                        _ => metadata.default = Some(quote! { #lit }),
                    }
                } else {
                    abort!(path, "duplicated default");
                }
            } else if path.is_ident("prompt") {
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
            } else {
                abort!(path, "unknown attribute");
            }
        }
        _ => abort!(meta, "unknown attribute"),
    }
}

/// Generate the struct that implements `BuildableValue` for the struct, and implement the `new()`
/// function for it.
fn gen_struct_builder(gen: &StructGenerator) -> TokenStream {
    let builder_ident = &gen.builder_ident;
    let fields_gen = gen.fields_def_list();
    let fields_new = gen.fields_new_list();
    let impl_debug = gen.impl_debug();
    quote! {
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        struct #builder_ident #fields_gen

        #impl_debug

        #[automatically_derived]
        #[allow(clippy::unnecessary_cast)]
        impl #builder_ident {
            fn new(config: ibuilder::BuildableValueConfig<()>) -> #builder_ident {
                #builder_ident #fields_new
            }
        }
    }
}

/// Generate the implementation of `NewBuildableValue` for the struct.
fn gen_impl_new_buildable_value(gen: &StructGenerator) -> TokenStream {
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
