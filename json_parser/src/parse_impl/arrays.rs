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

#[cfg(test)]
mod tests {
    use crate::json_value::JsonValue;

    use super::*;

    #[test]
    fn test_empty() {
        let result = Parser::parse::<Vec<JsonValue>>("[]");
        assert_eq!(Ok(Vec::new()), result);
    }

    #[test]
    fn test_float() {
        let expected_elems = vec![3.0, 5.6, 12e-4, 15.1];

        let result = Parser::parse::<Vec<f64>>(r#"[3.0, 5.6, 12e-4, 15.1]"#);
        assert_eq!(Ok(expected_elems), result);
    }

    #[test]
    fn test_nested() {
        let expected_elems = vec![vec![1, 2], vec![3, 4, 5]];

        let result = Parser::parse::<Vec<Vec<u32>>>(r#"[[1,2], [3, 4, 5]]"#);
        assert_eq!(Ok(expected_elems), result);
    }

    #[test]
    fn test_no_trailing_comma() {
        let result = Parser::parse::<Vec<f64>>(r#"[1, 2,]"#);
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnexpectedToken,
                line: 1,
                lexeme: ",".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_mixed() {
        let expected_elems = vec![
            JsonValue::String("first".to_string()),
            JsonValue::String("second".to_string()),
            JsonValue::Number(3.0),
            JsonValue::Bool(true),
        ];

        let result = Parser::parse::<Vec<JsonValue>>(r#"["first", "second", 3, true]"#);
        assert_eq!(Ok(expected_elems), result);
    }
}
