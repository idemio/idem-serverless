use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, PathArguments, Type, parse_macro_input};

#[proc_macro_derive(ConfigurableHandler)]
pub fn derive_init_function(input: TokenStream) -> TokenStream {
    use syn::{Data, DeriveInput, Fields, parse_macro_input};

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let struct_name = &input.ident;

    // Validate the struct has exactly one field
    let field_count = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named.named.len(),
            Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.len(),
            Fields::Unit => 0, // Unit struct has no fields
        },
        _ => {
            return TokenStream::from(
                syn::Error::new_spanned(input, "ConfigurableHandler can only be derived for structs.")
                    .to_compile_error(),
            );
        }
    };

    // Ensure the struct has exactly 1 field
    if field_count != 1 {
        return TokenStream::from(
            syn::Error::new_spanned(
                input,
                "ConfigurableHandler can only be derived for structs with exactly one field.",
            )
            .to_compile_error(),
        );
    }

    let mut inner_type = None;

    if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(fields_named) = data_struct.fields {
            for field in fields_named.named {
                inner_type = find_config_inner_type(field);
            }
        }
    }
    let inner_type = inner_type.expect("The struct must contain a field with type Config<X>.");

    let generated = quote! {
        impl #struct_name {
            pub fn init_handler(config: Config<#inner_type>) -> #struct_name {
                #struct_name {
                    config
                }
            }

            pub fn id() -> &'static str {
                stringify!(#struct_name)
            }

            pub fn config_file_name() -> &'static str {
                stringify!(#struct_name.json)
            }

            pub fn config(&self) -> &Config<#inner_type> {
                &self.config
            }

            pub fn reload_config(&mut self, config: Config<#inner_type>) {
                self.config = config;
            }
        }
    };
    TokenStream::from(generated)
}

fn find_config_inner_type(field: Field) -> Option<Type> {
    use syn::{PathArguments, Type, TypePath};

    if let Type::Path(TypePath { path, .. }) = &field.ty {
        // Get the last segment of the field.
        // i.e. idem-config::config::Config<X>, or Config<X>
        if let Some(segment) = path.segments.last() {
            // Make sure the outer type is 'Config' and grab the inner type argument
            if segment.ident == "Config" {
                if let PathArguments::AngleBracketed(type_argument) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = type_argument.args.first() {
                        return Some(inner.clone());
                    }
                }
            }
        }
    }
    None
}
