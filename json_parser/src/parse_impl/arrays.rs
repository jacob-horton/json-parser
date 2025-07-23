
use crate::{Parse, Parser, ParserErr, ParserErrKind, token::TokenKind};

impl<T: Parse> Parse for Vec<T> {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        parser.consume(TokenKind::LBracket)?;

        let mut elems = Vec::new();
        let mut had_comma = false;

        // Loop through all elements, until reaching closing bracket
        while !parser.check(TokenKind::RBracket)? {
            let elem = T::parse(parser)?;
            elems.push(elem);

            // Once no comma at end, we have reached end of array
            had_comma = parser.check(TokenKind::Comma)?;
            if had_comma {
                parser.advance()?;
            } else {
                break;
            }
        }

        // No trailing comma
        if had_comma {
            return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
        }

        parser.consume(TokenKind::RBracket)?;

        Ok(elems)
    }
}
