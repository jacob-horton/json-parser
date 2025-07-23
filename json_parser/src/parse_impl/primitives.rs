use std::str::FromStr;

use crate::{Parse, Parser, ParserErr, ParserErrKind, token::TokenKind};

// Define a trait so we can specify which number types we want to be parsable
pub trait JsonNumber: Sized + FromStr {}

impl JsonNumber for i128 {}
impl JsonNumber for i64 {}
impl JsonNumber for i32 {}
impl JsonNumber for i16 {}
impl JsonNumber for i8 {}

impl JsonNumber for u128 {}
impl JsonNumber for u64 {}
impl JsonNumber for u32 {}
impl JsonNumber for u16 {}
impl JsonNumber for u8 {}

impl JsonNumber for f64 {}
impl JsonNumber for f32 {}

impl<T: JsonNumber> Parse for T {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let token = parser.advance()?;
        match token.kind {
            TokenKind::Number => token
                .lexeme
                .parse::<T>()
                .map_err(|_| parser.make_err_prev(ParserErrKind::InvalidNumber)),
            _ => Err(parser.make_err_prev(ParserErrKind::UnexpectedToken)),
        }
    }
}

impl Parse for bool {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let token = parser.advance()?;
        if let TokenKind::Bool = token.kind {
            // NOTE: should only be "true" or "false", which is why we can do this
            Ok(token.lexeme == "true")
        } else {
            Err(parser.make_err_prev(ParserErrKind::UnexpectedToken))
        }
    }
}
