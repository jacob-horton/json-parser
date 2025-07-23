extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

fn derive_json_deserialise_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let fields = match &data.fields {
        Fields::Named(data) => data,
        _ => panic!(
            "JSON deserialising can only be derived for named field structs (no tuple or unit structs)"
        ),
    };

    let struct_name = &input.ident;

    // Code generation
    // fields_struct is a temporary object to store the field data when it's being parsed
    // Each value is initialised to None, and set once it is found
    let mut fields_struct_types = Vec::new();
    let mut fields_struct_init = Vec::new();

    // When we come across a property, set the value in the fields_struct
    // If the value does not exist in the fields_struct, report an error
    let mut field_setters = Vec::new();

    // Initialise the user's struct with the data collected
    // If there is a field missing, report an error
    let mut struct_init_lines = Vec::new();

    // Loop through each field
    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        // Generated code
        let field_type = quote! { #name: Option<#ty> };
        let field_init = quote! { #name: None };
        let field_setter =
            quote! { stringify!(#name) => parsed_fields.#name = Some(<#ty>::parse(parser)?), };
        let struct_init_line = quote! {
            #name: parsed_fields.#name.ok_or(
                parser.make_err_from_token(ParserErrKind::MissingProperty(stringify!(#name).to_string()), &l_curly_token)
            )?
        };

        // Add to vecs
        fields_struct_types.push(field_type);
        fields_struct_init.push(field_init);
        field_setters.push(field_setter);
        struct_init_lines.push(struct_init_line);
    }

    // Generated impl block
    let expanded = quote! {
        impl Parse for #struct_name {
            fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
                let l_curly_token = parser.consume(TokenKind::LCurlyBracket)?;

                let mut had_comma = false;

                // temporary object to store field data. Initialise all values to None
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
                        TokenKind::String(ref name) => {
                            parser.consume(TokenKind::Colon)?;

                            // Assign the data to the parsed_fields struct
                            match name.as_str() {
                                #(#field_setters)*
                                _ => return Err(parser.make_err_from_token(ParserErrKind::UnknownProperty, &token)),
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

                // Convert parsed_fields into the user's struct
                // If data is missing, return an error
                return Ok(#struct_name {
                    #(#struct_init_lines),*
                });
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(JsonDeserialise)]
pub fn derive_json_deserialise(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => return derive_json_deserialise_struct(&input, data),
        _ => panic!("Cannot derive JsonDeserialise on this type"),
    };
}
