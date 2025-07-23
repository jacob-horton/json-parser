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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object() {
        let result = Parser::parse::<JsonValue>(r#"{"prop": 3}"#);
        assert_eq!(
            Ok(JsonValue::Object(HashMap::from([(
                "prop".to_string(),
                JsonValue::Number(3.0)
            )]))),
            result
        );
    }

    #[test]
    fn test_array() {
        let result = Parser::parse::<JsonValue>(r#"[1, 2, 3]"#);
        assert_eq!(
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ])),
            result
        );
    }

    #[test]
    fn test_string() {
        let result = Parser::parse::<JsonValue>(r#""hi""#);
        assert_eq!(Ok(JsonValue::String("hi".to_string())), result);
    }

    #[test]
    fn test_number() {
        let result = Parser::parse::<JsonValue>(r#"5.55"#);
        assert_eq!(Ok(JsonValue::Number(5.55)), result);
    }

    #[test]
    fn test_bool() {
        let result = Parser::parse::<JsonValue>(r#"false"#);
        assert_eq!(Ok(JsonValue::Bool(false)), result);
    }

    #[test]
    fn test_null() {
        let result = Parser::parse::<JsonValue>(r#"null"#);
        assert_eq!(Ok(JsonValue::Null), result);
    }

    #[test]
    fn test_invalid_token() {
        let result = Parser::parse::<JsonValue>(r#":"#);
        assert_eq!(
            Err(ParserErr {
                kind: ParserErrKind::UnexpectedToken,
                line: 1,
                lexeme: ":".to_string(),
            }),
            result
        );
    }
}
