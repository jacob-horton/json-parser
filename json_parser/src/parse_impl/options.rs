use crate::{Parse, Parser, ParserErr, token::TokenKind};

impl<T: Parse> Parse for Option<T> {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        // If null, return `None`
        if parser.check(TokenKind::Null)? {
            parser.consume(TokenKind::Null)?;
            return Ok(None);
        }

        // Otherwise parse the inner type `T`
        Ok(Some(T::parse(parser)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::ParserErrKind;

    use super::*;

    #[test]
    fn test_null() {
        let result = Parser::parse::<Option<u32>>("null");
        assert_eq!(Ok(None), result);
    }

    #[test]
    fn test_nested_option_null() {
        let result = Parser::parse::<Option<Option<u32>>>("null");
        assert_eq!(Ok(None), result);
    }

    #[test]
    fn test_with_value() {
        let result = Parser::parse::<Option<String>>("\"42\"");
        assert_eq!(Ok(Some("42".to_string())), result);
    }

    #[test]
    fn test_nested_with_value() {
        let result = Parser::parse::<Option<Option<bool>>>("true");
        assert_eq!(Ok(Some(Some(true))), result);
    }

    #[test]
    fn test_nested_with_incorrect_type() {
        let result = Parser::parse::<Option<bool>>("5");
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnexpectedToken,
                line: 1,
                lexeme: "5".to_string(),
            }),
            result
        );
    }
}
