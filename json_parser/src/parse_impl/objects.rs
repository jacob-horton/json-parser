use std::collections::HashMap;

use crate::{Parse, Parser, ParserErr, ParserErrKind, token::TokenKind};

impl<T: Parse> Parse for HashMap<String, T> {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        parser.consume(TokenKind::LCurlyBracket)?;

        let mut props = HashMap::new();
        let mut had_comma = false;

        // Loop through all properties, until reaching closing bracket
        while !parser.check(TokenKind::RCurlyBracket)? {
            let token = parser.advance()?;
            match token.kind {
                TokenKind::String(name) => {
                    parser.consume(TokenKind::Colon)?;

                    let value = T::parse(parser)?;
                    props.insert(name, value);

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

        Ok(props)
    }
}
