use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(RepositoryMethods)]
pub fn check_field_type_derive(input: TokenStream) -> TokenStream {
    const FIELD_NAME: &str = "db_conn";
    const FIELD_TYPE: &str = "DatabaseConnection";

    // Parse the input into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = &input.ident;

    // Extract the fields of the struct
    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            &fields_named.named
        } else {
            return syn::Error::new_spanned(
                struct_name,
                "Only structs with named fields are supported",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(struct_name, "Only structs are supported")
            .to_compile_error()
            .into();
    };

    // Iterate over the fields to find the one named "db_conn"
    let field = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == FIELD_NAME);

    match field {
        // Field is not present
        None => {
            return syn::Error::new_spanned(
                struct_name,
                format!(
                    "The struct must have a field `{}` of type `{}`",
                    FIELD_NAME, FIELD_TYPE
                ),
            )
            .to_compile_error()
            .into();
        }
        // Check if field has correct type
        Some(f) => {
            if let Type::Path(type_path) = &f.ty {
                if !type_path.path.is_ident(FIELD_TYPE) {
                    let span = f.ty.span();
                    return syn::Error::new(
                        span,
                        format!(
                            "Expected: `{}` but found: `{}`",
                            FIELD_TYPE,
                            &f.ty.to_token_stream()
                        ),
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    // Generate the implementation
    let expanded = quote! {
        impl RepositoryMethods<Entity, ActiveModel> for #struct_name {
            fn db_conn(&self) -> &DatabaseConnection {
                &self.db_conn
            }
        }
    };

    TokenStream::from(expanded)
}
