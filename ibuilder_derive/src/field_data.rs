use proc_macro_error::abort;
use proc_macro_error::ResultExt;
use syn::export::TokenStream2;
use syn::punctuated::Punctuated;
use syn::{DataStruct, Field, Lit, Meta, MetaNameValue, Path, Token};

use quote::quote;

/// Metadata from a field of the derived structure.
#[derive(Debug)]
pub struct FieldData {
    /// Help message associated to the field.
    help: String,
    /// Default value for the field. This `TokenStream` follows this mapping:
    /// `#[ibuilder(default = XXX)]` => `Some(XXXX.into())` if the default attribute is present,
    /// otherwise it's just `None`.
    default: TokenStream2,
}

/// Get the `Ident` from the name of the field.
pub fn field_type(field: &Field) -> &Path {
    let ty = &field.ty;
    let path = if let syn::Type::Path(path) = ty {
        &path.path
    } else {
        abort!(ty, "Field type not supported");
    };
    path
}

/// Return the struct initializer items for all the fields of the structure. It gets expanded to
/// something like:
///
/// ```text
/// field1: something,
/// field2: something,
/// ...
/// ```
pub fn field_initializers(data: &DataStruct) -> TokenStream2 {
    let mut builder_init = Vec::new();
    for field in data.fields.iter() {
        let init = builder_initializer(field);
        builder_init.push(init);
    }
    quote! { #(#builder_init,)* }
}

/// Return the expression used to initialize the `BuildableValue` of the specified field.
pub fn builder_initializer(field: &Field) -> TokenStream2 {
    let field_type = field_type(field);
    let field_name = field.ident.as_ref().unwrap();
    let FieldData { help, default } = get_field_init_data(field);
    let segments = &field_type.segments;
    let is_auto = if segments.len() == 1 {
        match segments[0].ident.to_string().as_str() {
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize"
            | "f32" | "f64" | "String" | "char" | "bool" => true,
            _ => false,
        }
    } else {
        false
    };
    if is_auto {
        let ty = field_type_to_builder(&field_type);
        quote! { #field_name: Box::new(<#ty>::new(#default, #help.to_string())) }
    } else {
        quote! { #field_name: <#field_type>::new_builder(#help.to_string()) }
    }
}

/// Extract the initialization data for the builder or a field. It returns the help message and the
/// default value for the field.
pub fn get_field_init_data(field: &Field) -> FieldData {
    let mut help: Option<String> = None;
    let mut default: Option<Lit> = None;
    for att in &field.attrs {
        // rustdoc line starting with ///
        if att.path.is_ident("doc") {
            if help.is_none() {
                if let Ok(Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(s), ..
                })) = att.parse_meta()
                {
                    help = Some(s.value().trim().to_string());
                }
            }
        // attribute in the form: `#[ibuilder(...)]`
        } else if att.path.is_ident("ibuilder") {
            let meta = att
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .unwrap_or_abort();
            for meta in meta {
                match meta {
                    // #[ibuilder(path = lit)]
                    Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                        if path.is_ident("default") {
                            if default.is_none() {
                                default = Some(lit);
                            } else {
                                abort!(path, "Duplicated default");
                            }
                        } else {
                            abort!(path, "Unknown attribute");
                        }
                    }
                    // #[ibuilder(stuff)]
                    Meta::Path(path) => {
                        abort!(path, "Unknown attribute");
                    }
                    // #[ibuilder(stuff(lol))]
                    Meta::List(list) => {
                        abort!(list, "Unknown attribute");
                    }
                }
            }
        }
    }
    let help = help.unwrap_or_default();
    let default = match default {
        Some(def) => quote! { Some(#def.into()) },
        None => quote! { None },
    };
    FieldData { help, default }
}

/// Return the most suitable builder type for the field.
pub fn field_type_to_builder(field_type: &Path) -> TokenStream2 {
    let segments = &field_type.segments;
    if segments.len() != 1 {
        panic!("only builtin types are supported");
    }
    match segments[0].ident.to_string().as_str() {
        "i8" => quote! { ibuilder::builders::I8Builder },
        "i16" => quote! { ibuilder::builders::I16Builder },
        "i32" => quote! { ibuilder::builders::I32Builder },
        "i64" => quote! { ibuilder::builders::I64Builder },
        "u8" => quote! { ibuilder::builders::U8Builder },
        "u16" => quote! { ibuilder::builders::U16Builder },
        "u32" => quote! { ibuilder::builders::U32Builder },
        "u64" => quote! { ibuilder::builders::U64Builder },
        "isize" => quote! { ibuilder::builders::ISizeBuilder },
        "usize" => quote! { ibuilder::builders::USizeBuilder },
        "f32" => quote! { ibuilder::builders::F32Builder },
        "f64" => quote! { ibuilder::builders::F64Builder },
        "String" => quote! { ibuilder::builders::StringBuilder },
        "char" => quote! { ibuilder::builders::CharBuilder },
        "bool" => quote! { ibuilder::builders::BoolBuilder },
        ty => panic!("only builtin types are supported: {}", ty),
    }
}
