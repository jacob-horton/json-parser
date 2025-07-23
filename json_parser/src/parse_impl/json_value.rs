use crate::{Parse, Parser, ParserErr, ParserErrKind, TokenKind, json_value::JsonValue};
use std::collections::HashMap;

impl Parse for JsonValue {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let token = parser.peek()?;
        let ast = match token.kind {
            TokenKind::LCurlyBracket => Self::Object(<HashMap<String, JsonValue>>::parse(parser)?),
            TokenKind::LBracket => Self::Array(<Vec<JsonValue>>::parse(parser)?),
            TokenKind::String(_) => Self::String(String::parse(parser)?),
            TokenKind::Number => Self::Number(f64::parse(parser)?),
            TokenKind::Bool => Self::Bool(bool::parse(parser)?),
            TokenKind::Null => {
                parser.advance()?;
                Self::Null
            }
            _ => return Err(parser.make_err(ParserErrKind::UnexpectedToken)),
        };

        Ok(ast)
    }
}
