use std::{collections::HashMap, str::FromStr};

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
    UnrecognisedKeyword,
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

impl From<ScannerErr> for ParserErr {
    fn from(err: ScannerErr) -> Self {
        let kind = match err.kind {
            ScannerErrKind::UnexpectedEndOfSource => ParserErrKind::UnexpectedEndOfSource,
            ScannerErrKind::UnterminatedString => ParserErrKind::UnterminatedString,
            ScannerErrKind::UnrecognisedSymbol => ParserErrKind::UnrecognisedSymbol,
            ScannerErrKind::UnrecognisedKeyword => ParserErrKind::UnrecognisedKeyword,
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
            return Ok(self.advance()?);
        }

        Err(self.make_err(ParserErrKind::ExpectedToken(kind)))
    }

    pub fn check(&self, kind: TokenKind) -> Result<bool, ParserErr> {
        Ok(self.peek()?.kind == kind)
    }

    fn peek(&self) -> Result<Token, ParserErr> {
        self.current
            .clone()
            .ok_or(self.make_err(ParserErrKind::UnexpectedEndOfSource))
    }

    pub fn advance(&mut self) -> Result<Token, ParserErr> {
        self.prev = self.current.clone();
        self.current = self.scanner.next_token()?;

        Ok(self.previous())
    }

    fn previous(&self) -> Token {
        self.prev.clone().expect(BUG_PREV_BEFORE_ADVANCE)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),

    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

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

impl Parse for String {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        if let TokenKind::String(val) = parser.advance()?.kind {
            Ok(val)
        } else {
            Err(parser.make_err_prev(ParserErrKind::UnexpectedToken))
        }
    }
}

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

impl<T: Parse> Parse for Option<T> {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        match parser.peek()?.kind {
            TokenKind::Null => {
                parser.consume(TokenKind::Null)?;
                Ok(None)
            }
            _ => Ok(Some(T::parse(parser)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub props: HashMap<String, JsonValue>,
}

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

// TODO: tests for specific types (not just Any)
#[cfg(test)]
mod tests {
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
            ("{ some_prop: 5 }", ParserErrKind::UnrecognisedKeyword),
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
            ("tru", ParserErrKind::UnrecognisedKeyword),
            ("nulll", ParserErrKind::UnrecognisedKeyword),
            ("[--1]", ParserErrKind::InvalidNumber),
            ("[+1]", ParserErrKind::UnrecognisedSymbol),
            (r#"{null: "value"}"#, ParserErrKind::UnexpectedToken),
            (r#"{"key": undefined}"#, ParserErrKind::UnrecognisedKeyword),
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
