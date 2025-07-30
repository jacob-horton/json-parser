use crate::{Parse, Parser, ParserErr, ParserErrKind, token::TokenKind};

impl Parse for String {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        // If we have a string, return the value captured by the scanner
        // Otherwise, we expected a string, but didn't get one - error
        match parser.advance()?.kind {
            TokenKind::String(val) => Ok(val),
            _ => Err(parser.make_err_prev(ParserErrKind::UnexpectedToken)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParserErrKind;

    use super::*;

    #[test]
    fn test_unquoted() {
        let result = Parser::parse::<String>(r#""test""#);
        assert_eq!(Ok("test".to_string()), result);
    }

    #[test]
    fn test_valid_escape_sequences() {
        let cases = vec![
            (r#""\u00A9""#, "©"),
            (r#""\n""#, "\n"),
            (r#""\r""#, "\r"),
            (r#""\b""#, "\x08"),
            (r#""\/""#, "/"),
            (r#""\\""#, "\\"),
        ];

        for (source, expected) in cases {
            let result = Parser::parse::<String>(source);
            assert_eq!(Ok(expected.to_string()), result);
        }
    }

    #[test]
    fn test_invalid_escape_sequences() {
        let cases = vec![
            (r#""\uZZZZ""#, r#""\uZZZZ"#),
            (r#""\uD800""#, r#""\uD800"#),
            (r#""bad\escape""#, r#""bad\e"#),
        ];

        for (source, error_lexeme) in cases {
            let result = Parser::parse::<String>(source);
            assert_eq!(
                Err(ParserErr {
                    kind: ParserErrKind::InvalidEscapeSequence,
                    line: 1,
                    lexeme: error_lexeme.to_string(),
                }),
                result
            );
        }
    }
}
