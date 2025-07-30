use crate::{
    scanner::{Scanner, ScannerErr, ScannerErrKind},
    token::{Token, TokenKind},
};

static BUG_PREV_BEFORE_ADVANCE: &str = "[BUG] Called `prev` before advancing - no previous value";
static BUG_NO_TOKEN_ERR_REPORT: &str = "[BUG] Failed to get token for reporting error";

#[derive(Debug, Clone, PartialEq)]
pub struct ParserErr {
    pub kind: ParserErrKind,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrKind {
    // Scanner specific errors
    UnterminatedString,
    UnrecognisedSymbol,
    UnrecognisedLiteral,
    InvalidNumber,
    InvalidEscapeSequence,

    // Parser specific errors
    ExpectedEndOfSource,
    ExpectedToken(TokenKind),
    UnexpectedToken,
    UnknownProperty,
    MissingProperty(String),

    // Both
    UnexpectedEndOfSource,
}

// Convert ScannerErr to ParserErr (easy 1 to 1 mapping)
impl From<ScannerErr> for ParserErr {
    fn from(err: ScannerErr) -> Self {
        let kind = match err.kind {
            ScannerErrKind::UnexpectedEndOfSource => ParserErrKind::UnexpectedEndOfSource,
            ScannerErrKind::UnterminatedString => ParserErrKind::UnterminatedString,
            ScannerErrKind::UnrecognisedSymbol => ParserErrKind::UnrecognisedSymbol,
            ScannerErrKind::UnrecognisedLiteral => ParserErrKind::UnrecognisedLiteral,
            ScannerErrKind::InvalidNumber => ParserErrKind::InvalidNumber,
            ScannerErrKind::InvalidEscapeSequence => ParserErrKind::InvalidEscapeSequence,
        };

        Self {
            line: err.line,
            lexeme: err.lexeme,
            kind,
        }
    }
}

pub trait Parse {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    scanner: Scanner<'a>,

    prev: Option<Token>,
    current: Option<Token>,
}

impl Parser<'_> {
    pub fn make_err(&self, kind: ParserErrKind) -> ParserErr {
        // Get current token, fallback to previous
        let err_token = self
            .current
            .clone()
            .unwrap_or_else(|| self.prev.clone().expect(BUG_NO_TOKEN_ERR_REPORT));

        ParserErr {
            kind,
            line: err_token.line,
            lexeme: err_token.lexeme,
        }
    }

    pub fn make_err_from_token(&self, kind: ParserErrKind, token: &Token) -> ParserErr {
        ParserErr {
            kind,
            line: token.line,
            lexeme: token.lexeme.to_owned(),
        }
    }

    // Make err with prev token instead of current
    pub fn make_err_prev(&self, kind: ParserErrKind) -> ParserErr {
        let err_token = self.prev.clone().expect(BUG_NO_TOKEN_ERR_REPORT);

        ParserErr {
            kind,
            line: err_token.line,
            lexeme: err_token.lexeme,
        }
    }

    pub fn parse<T: Parse>(source: &str) -> Result<T, ParserErr> {
        let mut scanner = Scanner::init(source);
        let current = scanner.next_token()?;

        let mut parser = Parser {
            scanner,
            current,
            prev: None,
        };

        let result = T::parse(&mut parser)?;
        if parser.current.is_some() {
            return Err(parser.make_err(ParserErrKind::ExpectedEndOfSource));
        }

        Ok(result)
    }

    pub fn consume(&mut self, kind: TokenKind) -> Result<Token, ParserErr> {
        if self.check(kind.clone())? {
            return self.advance();
        }

        Err(self.make_err(ParserErrKind::ExpectedToken(kind)))
    }

    pub fn check(&self, kind: TokenKind) -> Result<bool, ParserErr> {
        Ok(self.peek()?.kind == kind)
    }

    pub fn peek(&self) -> Result<Token, ParserErr> {
        self.current
            .clone()
            .ok_or(self.make_err(ParserErrKind::UnexpectedEndOfSource))
    }

    pub fn advance(&mut self) -> Result<Token, ParserErr> {
        self.prev = self.current.clone();
        self.current = self.scanner.next_token()?;

        Ok(self.previous())
    }

    pub fn previous(&self) -> Token {
        self.prev.clone().expect(BUG_PREV_BEFORE_ADVANCE)
    }
}

#[cfg(test)]
mod tests {
    use crate::json_value::JsonValue;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_top_level() {
        let cases = vec![
            ("[]", JsonValue::Array(vec![])),
            ("{}", JsonValue::Object(HashMap::new())),
            ("1234", JsonValue::Number(1234.0)),
            ("1234e5", JsonValue::Number(1234e5)),
            ("1234.567", JsonValue::Number(1234.567)),
            ("1234.567e5", JsonValue::Number(1234.567e5)),
            (r#""str a_b""#, JsonValue::String("str a_b".to_string())),
            ("true", JsonValue::Bool(true)),
            ("false", JsonValue::Bool(false)),
            ("null", JsonValue::Null),
        ];

        for (source, expected) in cases {
            let result = Parser::parse(source);
            assert_eq!(Ok(expected), result);
        }
    }

    #[test]
    fn test_object() {
        let expected_props = HashMap::from([
            (
                "name".to_string(),
                JsonValue::String("Jane Doe".to_string()),
            ),
            ("age".to_string(), JsonValue::Number(32.0)),
        ]);
        let result = Parser::parse(r#"{"name": "Jane Doe", "age": 32}"#);
        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(expected_props, obj);
        } else {
            panic!("Expected 'Ok(Any::Object(_))', got '{result:?}'");
        }
    }

    #[test]
    fn test_array() {
        let expected_elems = vec![
            JsonValue::String("first".to_string()),
            JsonValue::String("second".to_string()),
            JsonValue::Number(3.0),
            JsonValue::Bool(true),
        ];

        let result = Parser::parse(r#"["first", "second", 3, true]"#);
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(expected_elems, arr);
        } else {
            panic!("Expected 'Ok(Any::Object(_))', got '{result:?}'");
        }
    }

    #[test]
    fn test_invalid_json() {
        let cases = vec![
            ("[,]", ParserErrKind::UnexpectedToken),
            ("{", ParserErrKind::UnexpectedEndOfSource),
            ("{} []", ParserErrKind::ExpectedEndOfSource),
            ("1234a", ParserErrKind::InvalidNumber),
            (r#"["trailing", "comma",]"#, ParserErrKind::UnexpectedToken),
            (r#"{"trailing": "comma",}"#, ParserErrKind::UnexpectedToken),
            (
                r#"["no" "comma"]"#,
                ParserErrKind::ExpectedToken(TokenKind::RBracket),
            ),
            ("{ true: 5 }", ParserErrKind::UnexpectedToken),
            ("{ 10: 5 }", ParserErrKind::UnexpectedToken),
            ("{ some_prop: 5 }", ParserErrKind::UnrecognisedLiteral),
            ("^", ParserErrKind::UnrecognisedSymbol),
            (r#""unclosed string"#, ParserErrKind::UnexpectedEndOfSource),
            (
                "[1, 2 3]",
                ParserErrKind::ExpectedToken(TokenKind::RBracket),
            ),
            (
                r#"{"key" "value"}"#,
                ParserErrKind::ExpectedToken(TokenKind::Colon),
            ),
            (r#"{"key": "value""#, ParserErrKind::UnexpectedEndOfSource),
            ("[null,]", ParserErrKind::UnexpectedToken),
            (r#"{"a": null,}"#, ParserErrKind::UnexpectedToken),
            ("tru", ParserErrKind::UnrecognisedLiteral),
            ("nulll", ParserErrKind::UnrecognisedLiteral),
            ("[--1]", ParserErrKind::InvalidNumber),
            ("[+1]", ParserErrKind::UnrecognisedSymbol),
            (r#"{null: "value"}"#, ParserErrKind::UnexpectedToken),
            (r#"{"key": undefined}"#, ParserErrKind::UnrecognisedLiteral),
            (r#""\uZZZZ""#, ParserErrKind::InvalidEscapeSequence),
            (
                r#"{"\uD800": "high surrogate only"}"#,
                ParserErrKind::InvalidEscapeSequence,
            ),
            (r#""bad\escape""#, ParserErrKind::InvalidEscapeSequence),
        ];

        for (source, expected) in cases {
            let result = Parser::parse::<JsonValue>(source);
            assert_eq!(
                Err(expected),
                result.map_err(|x| x.kind),
                "Ensure the following JSON is invalid: {source}"
            );
        }
    }
}
