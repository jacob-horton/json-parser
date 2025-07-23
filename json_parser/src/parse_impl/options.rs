
use crate::{Parse, Parser, ParserErr, token::TokenKind};

impl<T: Parse> Parse for Option<T> {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        match parser.peek()?.kind {
            TokenKind::Null => {
                parser.consume(TokenKind::Null)?;
                Ok(None)
            }
            _ => Ok(Some(T::parse(parser)?)),
        }
    }
}
