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

#[cfg(test)]
mod tests {
    use crate::json_value::JsonValue;

    use super::*;

    #[test]
    fn test_empty() {
        let result = Parser::parse::<HashMap<String, JsonValue>>("{}");
        assert_eq!(Ok(HashMap::new()), result);
    }

    #[test]
    fn test_float_only() {
        let result =
            Parser::parse::<HashMap<String, f64>>(r#"{"prop1": 5, "prop2": 3e2, "prop3": 16.9}"#);

        let expected = HashMap::from([
            ("prop1".to_string(), 5.0),
            ("prop2".to_string(), 3e2),
            ("prop3".to_string(), 16.9),
        ]);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn test_mixed() {
        let result = Parser::parse::<HashMap<String, JsonValue>>(
            r#"{"prop1": 5, "prop2": true, "prop3": "test"}"#,
        );

        let expected = HashMap::from([
            ("prop1".to_string(), JsonValue::Number(5.0)),
            ("prop2".to_string(), JsonValue::Bool(true)),
            ("prop3".to_string(), JsonValue::String("test".to_string())),
        ]);

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn test_prop_no_quotes() {
        let result = Parser::parse::<HashMap<String, JsonValue>>(r#"{prop: 5}"#);
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnrecognisedLiteral,
                line: 1,
                lexeme: "prop".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_literal_prop_key() {
        let result = Parser::parse::<HashMap<String, JsonValue>>(r#"{true: 5}"#);
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnexpectedToken,
                line: 1,
                lexeme: "true".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_no_trailing_comma() {
        let result = Parser::parse::<HashMap<String, JsonValue>>(r#"{"prop1": 5, "prop2": true,}"#);
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
    fn test_missing_colon() {
        let result = Parser::parse::<HashMap<String, JsonValue>>(r#"{"prop" 5}"#);
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::ExpectedToken(TokenKind::Colon),
                line: 1,
                lexeme: "5".to_string(),
            }),
            result
        );
    }

    #[test]
    fn test_nested() {
        let expected_props = HashMap::from([
            (
                "name".to_string(),
                JsonValue::String("Jane Doe".to_string()),
            ),
            (
                "nested".to_string(),
                JsonValue::Object(HashMap::from([
                    ("age".to_string(), JsonValue::Number(32.0)),
                    (
                        "phone".to_string(),
                        JsonValue::String("01234567890".to_string()),
                    ),
                ])),
            ),
        ]);

        let result = Parser::parse::<HashMap<String, JsonValue>>(
            r#"{"name": "Jane Doe", "nested": {"age": 32, "phone": "01234567890"}}"#,
        );
        assert_eq!(Ok(expected_props), result);
    }
}
