extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

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

    let fields_struct_types = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;

        quote! {
            #field_name: Option<#field_type>
        }
    });

    let fields_struct_init = fields.named.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        quote! {
            #ident: None
        }
    });

    let field_branches = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;

        quote! {
            stringify!(#field_name) => parsed_fields.#field_name = Some(<#field_type>::parse(parser)?),
        }
    });

    let constructor_fields = fields.named.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();

        quote! {#field_name: parsed_fields.#field_name.expect(&format!("Missing field: {}", stringify!(#field_name)))}
    });

    // TODO: properly handle errors
    let expanded = quote! {
        impl Parse for #struct_name {
            fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
                parser.consume(TokenKind::LCurlyBracket)?;

                let mut had_comma = false;

                let mut parsed_fields = {
                    struct ParsedFields {
                        #( #fields_struct_types, )*
                    }

                    ParsedFields {
                        #( #fields_struct_init, )*
                    }
                };

                // Loop through all properties, until reaching closing bracket
                while !parser.check(TokenKind::RCurlyBracket)? {
                    let token = parser.advance()?;
                    match token.kind {
                        TokenKind::String(name) => {
                            parser.consume(TokenKind::Colon)?;

                            // Assign variables
                            match name.as_str() {
                                #(#field_branches)*
                                _ => panic!("Unknown field: {name}"),
                            };

                            // Once no comma at end, we have reached end of object
                            had_comma = parser.check(TokenKind::Comma)?;
                            if had_comma {
                                parser.advance()?;
                            } else {
                                break;
                            }
                        }
                        _ => return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken)),
                    }
                }

                // No trailing comma
                if had_comma {
                    return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
                }

                parser.consume(TokenKind::RCurlyBracket)?;

                return Ok(#struct_name {
                    #(#constructor_fields),*
                });
            }
        }
    };

    expanded.into()
}
