use std::collections::HashMap;

use crate::{
    scanner::{Scanner, ScannerErr, ScannerErrKind},
    token::{Token, TokenKind},
};

static BUG_PREV_BEFORE_ADVANCE: &'static str =
    "[BUG] Called `prev` before advancing - no previous value";
static BUG_NO_TOKEN_ERR_REPORT: &'static str = "[BUG] Failed to get token for reporting error";

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

        return Self {
            line: err.line,
            lexeme: err.lexeme,
            kind,
        };
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

impl<'a> Parser<'a> {
    fn make_err(&self, kind: ParserErrKind) -> ParserErr {
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

    // TODO: make all these private somehow

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

    pub fn consume(&mut self, kind: TokenKind) -> Result<(), ParserErr> {
        if self.check(kind.clone())? {
            self.advance()?;
            return Ok(());
        }

        return Err(self.make_err(ParserErrKind::ExpectedToken(kind)));
    }

    pub fn check(&self, kind: TokenKind) -> Result<bool, ParserErr> {
        return Ok(self.peek()?.kind == kind);
    }

    fn peek(&self) -> Result<Token, ParserErr> {
        return self
            .current
            .clone()
            .ok_or(self.make_err(ParserErrKind::UnexpectedEndOfSource));
    }

    pub fn advance(&mut self) -> Result<Token, ParserErr> {
        self.prev = self.current.clone();
        self.current = self.scanner.next_token()?;

        return Ok(self.previous());
    }

    fn previous(&self) -> Token {
        return self.prev.clone().expect(BUG_PREV_BEFORE_ADVANCE);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Any {
    Object(Object),
    Array(Array),

    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl Parse for Any {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let token = parser.peek()?;
        let ast = match token.kind {
            TokenKind::LCurlyBracket => Self::Object(Object::parse(parser)?),
            TokenKind::LBracket => Self::Array(Array::parse(parser)?),
            TokenKind::String(_) => Self::String(String::parse(parser)?),
            TokenKind::Number(_) => Self::Number(f64::parse(parser)?),
            TokenKind::Bool(_) => Self::Bool(bool::parse(parser)?),
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
            return Ok(val);
        } else {
            return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
        }
    }
}

impl Parse for f64 {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        if let TokenKind::Number(val) = parser.advance()?.kind {
            return Ok(val);
        } else {
            return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
        }
    }
}

impl Parse for i64 {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        if let TokenKind::Number(val) = parser.advance()?.kind {
            if val.fract() != 0.0 {
                panic!("Cannot convert float to int");
            }

            return Ok(val as i64);
        } else {
            return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
        }
    }
}

impl Parse for bool {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        if let TokenKind::Bool(val) = parser.advance()?.kind {
            return Ok(val);
        } else {
            return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken));
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
                return Ok(None);
            }
            _ => Ok(Some(T::parse(parser)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub props: HashMap<String, Any>,
}

impl Parse for Object {
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

                    let value = Any::parse(parser)?;
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

        Ok(Self { props })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub elems: Vec<Any>,
}

impl Parse for Array {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        parser.consume(TokenKind::LBracket)?;

        let mut elems = Vec::new();
        let mut had_comma = false;

        // Loop through all elements, until reaching closing bracket
        while !parser.check(TokenKind::RBracket)? {
            let elem = Any::parse(parser)?;
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

        Ok(Self { elems })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level() {
        let cases = vec![
            ("[]", Any::Array(Array { elems: vec![] })),
            (
                "{}",
                Any::Object(Object {
                    props: HashMap::new(),
                }),
            ),
            ("1234", Any::Number(1234.0)),
            ("1234e5", Any::Number(1234e5)),
            ("1234.567", Any::Number(1234.567)),
            ("1234.567e5", Any::Number(1234.567e5)),
            (r#""str a_b""#, Any::String("str a_b".to_string())),
            ("true", Any::Bool(true)),
            ("false", Any::Bool(false)),
            ("null", Any::Null),
        ];

        for (source, expected) in cases {
            let result = Parser::parse(source);
            assert_eq!(Ok(expected), result);
        }
    }

    #[test]
    fn test_object() {
        let expected_props = HashMap::from([
            ("name".to_string(), Any::String("Jane Doe".to_string())),
            ("age".to_string(), Any::Number(32.0)),
        ]);
        let result = Parser::parse(r#"{"name": "Jane Doe", "age": 32}"#);
        if let Ok(Any::Object(obj)) = result {
            assert_eq!(expected_props, obj.props);
        } else {
            panic!("Expected 'Ok(Any::Object(_))', got '{result:?}'");
        }
    }

    #[test]
    fn test_array() {
        let expected_elems = vec![
            Any::String("first".to_string()),
            Any::String("second".to_string()),
            Any::Number(3.0),
            Any::Bool(true),
        ];

        let result = Parser::parse(r#"["first", "second", 3, true]"#);
        if let Ok(Any::Array(arr)) = result {
            assert_eq!(expected_elems, arr.elems);
        } else {
            panic!("Expected 'Ok(Any::Object(_))', got '{result:?}'");
        }
    }

    #[test]
    fn test_everything() {
        let source = include_str!("test_data/test_blob.json");
        let result = Parser::parse(source);

        let expected = Any::Object(Object {
            props: HashMap::from([
                ("name".to_string(), Any::String("Jane Doe".to_string())),
                ("age".to_string(), Any::Number(32.0)),
                ("is_verified".to_string(), Any::Bool(true)),
                ("balance".to_string(), Any::Number(10457.89)),
                ("nickname".to_string(), Any::Null),
                (
                    "contact".to_string(),
                    Any::Object(Object {
                        props: HashMap::from([
                            (
                                "email".to_string(),
                                Any::String("jane.doe@example.com".to_string()),
                            ),
                            (
                                "phone".to_string(),
                                Any::String("+1-555-123-4567".to_string()),
                            ),
                            (
                                "address".to_string(),
                                Any::Object(Object {
                                    props: HashMap::from([
                                        (
                                            "street".to_string(),
                                            Any::String("123 Maple Street".to_string()),
                                        ),
                                        (
                                            "city".to_string(),
                                            Any::String("Springfield".to_string()),
                                        ),
                                        ("zipcode".to_string(), Any::String("12345".to_string())),
                                        ("country".to_string(), Any::String("USA".to_string())),
                                    ]),
                                }),
                            ),
                        ]),
                    }),
                ),
                (
                    "preferences".to_string(),
                    Any::Object(Object {
                        props: HashMap::from([
                            (
                                "notifications".to_string(),
                                Any::Object(Object {
                                    props: HashMap::from([
                                        ("email".to_string(), Any::Bool(true)),
                                        ("sms".to_string(), Any::Bool(false)),
                                    ]),
                                }),
                            ),
                            ("theme".to_string(), Any::String("dark".to_string())),
                            ("language".to_string(), Any::String("en-US".to_string())),
                        ]),
                    }),
                ),
                (
                    "tags".to_string(),
                    Any::Array(Array {
                        elems: vec![
                            Any::String("user".to_string()),
                            Any::String("admin".to_string()),
                            Any::String("editor".to_string()),
                        ],
                    }),
                ),
                (
                    "history".to_string(),
                    Any::Array(Array {
                        elems: vec![
                            Any::Object(Object {
                                props: HashMap::from([
                                    (
                                        "login".to_string(),
                                        Any::String("2025-07-01T12:34:56Z".to_string()),
                                    ),
                                    ("ip".to_string(), Any::String("192.168.1.1".to_string())),
                                    ("success".to_string(), Any::Bool(true)),
                                ]),
                            }),
                            Any::Object(Object {
                                props: HashMap::from([
                                    (
                                        "login".to_string(),
                                        Any::String("2025-06-30T08:21:12Z".to_string()),
                                    ),
                                    ("ip".to_string(), Any::String("192.168.1.2".to_string())),
                                    ("success".to_string(), Any::Bool(false)),
                                ]),
                            }),
                        ],
                    }),
                ),
                (
                    "unicode_example".to_string(),
                    Any::String(
                        "Emoji test: 😄, 中文, العربية, escape test: \\n\\t\\\"\n".to_string(),
                    ),
                ),
                (
                    "numbers".to_string(),
                    Any::Object(Object {
                        props: HashMap::from([
                            ("int".to_string(), Any::Number(42.0)),
                            ("float".to_string(), Any::Number(3.14159)),
                            ("scientific".to_string(), Any::Number(6.022e23)),
                            ("scientific_no_decimal".to_string(), Any::Number(6e5)),
                            ("negative".to_string(), Any::Number(-5.0)),
                            ("negative_scientific".to_string(), Any::Number(-5.1e-10)),
                        ]),
                    }),
                ),
            ]),
        });

        assert_eq!(Ok(expected), result);
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
            let result = Parser::parse::<Any>(source);
            assert_eq!(
                Err(expected),
                result.map_err(|x| x.kind),
                "Ensure the following JSON is invalid: {source}"
            );
        }
    }
}
