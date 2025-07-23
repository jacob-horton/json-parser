
use crate::{Parse, Parser, ParserErr, ParserErrKind, token::TokenKind};

impl Parse for String {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        if let TokenKind::String(val) = parser.advance()?.kind {
            Ok(val)
        } else {
            Err(parser.make_err_prev(ParserErrKind::UnexpectedToken))
        }
    }
}
