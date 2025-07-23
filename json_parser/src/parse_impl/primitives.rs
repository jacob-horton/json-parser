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

#[cfg(test)]
mod tests {
    use crate::ParserErrKind;

    use super::*;

    #[test]
    fn test_signed_int() {
        let result = Parser::parse::<i64>("-5");
        assert_eq!(Ok(-5), result);
    }

    #[test]
    fn test_unsigned_int() {
        let result = Parser::parse::<u16>("5");
        assert_eq!(Ok(5), result);
    }

    #[test]
    fn test_unsigned_int_negative() {
        let result = Parser::parse::<u32>("-5");
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::InvalidNumber,
                line: 1,
                lexeme: "-5".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_float() {
        let result = Parser::parse::<f32>("-5.1");
        assert_eq!(Ok(-5.1), result);
    }

    #[test]
    fn test_scientific_notation() {
        let result = Parser::parse::<f32>("-5.1e-2");
        assert_eq!(Ok(-5.1e-2), result);
    }

    #[test]
    fn test_int_scientific_notation() {
        let result = Parser::parse::<i32>("5e2");
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::InvalidNumber,
                line: 1,
                lexeme: "5e2".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_bool() {
        let result = Parser::parse::<bool>("true");
        assert_eq!(Ok(true), result);

        let result = Parser::parse::<bool>("false");
        assert_eq!(Ok(false), result);
    }

    #[test]
    fn test_bool_invalid() {
        let result = Parser::parse::<bool>("null");
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnexpectedToken,
                line: 1,
                lexeme: "null".to_string(),
            }),
            result
        );
    }
}
