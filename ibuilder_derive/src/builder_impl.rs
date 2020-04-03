use syn::export::TokenStream2;
use syn::Ident;

use quote::quote;

/// Generate the `TokenStream` for an `impl` block that implements the `builder()` function for
/// constructing a `Builder<T>` and the `NewBuildableValue` impl for the base type.
///
/// It will generate a block similar to:
/// ```
/// # use ibuilder::{Options, ChooseError, nodes::Node};
/// # use std::any::Any;
/// # #[derive(Debug)] struct __Example_BuildableValueImpl;
/// # impl __Example_BuildableValueImpl { fn new(_: String) -> Self { unimplemented!() }}
/// # impl ibuilder::BuildableValue for __Example_BuildableValueImpl {
/// #     fn get_help(&self) -> &str { unimplemented!() }
/// #     fn apply(&mut self,data: &str,current_fields: &[String]) -> Result<(), ChooseError> { unimplemented!() }
/// #     fn get_options(&self,current_fields: &[String]) -> Options { unimplemented!() }
/// #     fn get_subfields(&self,current_fields: &[String]) -> Vec<String> { unimplemented!() }
/// #     fn to_node(&self) -> Node { unimplemented!() }
/// #     fn get_value_any(&self) -> Option<Box<dyn Any>> { unimplemented!()}
/// # }
/// # struct Example;
/// // struct Example;
///
/// #[automatically_derived]
/// impl Example {
///     pub fn builder() -> ibuilder::Builder<Example> {
///         # unimplemented!()
///         // ...
///     }
/// }
///
/// #[automatically_derived]
/// impl ibuilder::NewBuildableValue for Example {
///     fn new_builder(help: String) -> Box<dyn ibuilder::BuildableValue> {
///         Box::new(__Example_BuildableValueImpl::new(help))
///     }
/// }
/// ```
pub fn gen_builder_gen_method(struct_builder_name: &Ident, name: &Ident) -> TokenStream2 {
    quote! {
        #[automatically_derived]
        impl #name {
            pub fn builder() -> ibuilder::Builder<#name> {
                ibuilder::Builder::from_buildable_value(<#name>::new_builder("".to_string()))
            }
        }

        #[automatically_derived]
        impl ibuilder::NewBuildableValue for #name {
            fn new_builder(help: String) -> Box<dyn ibuilder::BuildableValue> {
                Box::new(#struct_builder_name::new(help))
            }
        }
    }
}
