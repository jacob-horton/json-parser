extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

// TODO: try generating parsing code from struct instead of validating

#[proc_macro_derive(JsonDeserialise)]
pub fn derive_json_deserialise(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // TODO: support enums
    let schema = if let Data::Struct(data) = &input.data {
        data
    } else {
        return Error::new_spanned(&input, "JSON deserialising can only be derived for structs")
            .to_compile_error()
            .into();
    };

    // TODO: support other structs
    let fields = if let Fields::Named(data) = &schema.fields {
        data
    } else {
        return syn::Error::new_spanned(
            &input,
            "JSON deserialising can only be derived for named field structs",
        )
        .to_compile_error()
        .into();
    };

    let struct_name = input.ident;
    let field_debugs = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;

        quote! {
            println!("Field: {:?}, Type: {:?}", stringify!(#field_name), stringify!(#field_type));
        }
    });

    let prop_validators = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_value = 32u64;

        quote! {#field_name: #field_value}
    });

    let constructor = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_value = 32u64;

        quote! {#field_name: #field_value}
    });

    // TODO: properly handle errors
    let expanded = quote! {
        impl #struct_name {
            fn parse_json(source: &str) -> Result<Self, ParserErr> {
                let parsed = Parser::parse(source).unwrap();
                let obj = match parsed {
                    Any::Object(data) => data,
                    _ => panic!("Not an object"),
                };
                println!("{obj:?}");

                #(#field_debugs)*

                return Ok(#struct_name {
                    #(#constructor),*
                });
            }
        }
    };

    expanded.into()
}
