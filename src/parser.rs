use std::collections::HashMap;

use crate::{
    scanner::{Scanner, ScannerErr, ScannerErrKind},
    token::{Token, TokenKind},
};

static BUG_PREV_BEFORE_ADVANCE: &'static str =
    "[BUG] Called `prev` before advancing - no previous value";
static BUG_NO_TOKEN_ERR_REPORT: &'static str = "[BUG] Failed to get token for reporting error";

#[derive(Debug, Clone)]
pub struct ParserErr {
    pub kind: ParserErrKind,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug, Clone)]
pub enum ParserErrKind {
    // Scanner specific errors
    UnterminatedString,
    UnrecognisedSymbol,
    UnrecognisedKeyword,
    InvalidNumber,

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
        };

        return Self {
            line: err.line,
            lexeme: err.lexeme,
            kind,
        };
    }
}

trait Parse {
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

    // Make err with prev token instead of current
    fn make_err_prev(&self, kind: ParserErrKind) -> ParserErr {
        let err_token = self.prev.clone().expect(BUG_NO_TOKEN_ERR_REPORT);

        ParserErr {
            kind,
            line: err_token.line,
            lexeme: err_token.lexeme,
        }
    }

    pub fn parse(source: &str) -> Result<Any, ParserErr> {
        let mut scanner = Scanner::init(source);
        let current = scanner.next_token()?;

        let mut parser = Parser {
            scanner,
            current,
            prev: None,
        };

        let result = Any::parse(&mut parser)?;
        if parser.current.is_some() {
            return Err(parser.make_err(ParserErrKind::ExpectedEndOfSource));
        }

        Ok(result)
    }

    fn consume(&mut self, kind: TokenKind) -> Result<(), ParserErr> {
        if self.check(kind.clone())? {
            self.advance()?;
            return Ok(());
        }

        return Err(self.make_err(ParserErrKind::ExpectedToken(kind)));
    }

    fn check(&self, kind: TokenKind) -> Result<bool, ParserErr> {
        return Ok(self.peek()?.kind == kind);
    }

    fn peek(&self) -> Result<Token, ParserErr> {
        return self
            .current
            .clone()
            .ok_or(self.make_err(ParserErrKind::UnexpectedEndOfSource));
    }

    fn advance(&mut self) -> Result<Token, ParserErr> {
        self.prev = self.current.clone();
        self.current = self.scanner.next_token()?;

        return Ok(self.previous());
    }

    fn previous(&self) -> Token {
        return self.prev.clone().expect(BUG_PREV_BEFORE_ADVANCE);
    }
}

#[derive(Debug, Clone)]
pub enum Any {
    Object(Object),
    Array(Array),

    String(String),
    Float(f64),
    Int(i64),
    Boolean(bool),
    Null,
}

impl Parse for Any {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let token = parser.advance()?;
        let ast = match token.kind {
            TokenKind::LCurlyBracket => Self::Object(Object::parse(parser)?),
            TokenKind::LBracket => Self::Array(Array::parse(parser)?),
            TokenKind::String(x) => Self::String(x),
            TokenKind::Float(x) => Self::Float(x),
            TokenKind::Int(x) => Self::Int(x),
            TokenKind::Boolean(x) => Self::Boolean(x),
            TokenKind::Null => Self::Null,
            _ => return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken)),
        };

        Ok(ast)
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    props: HashMap<String, Any>,
}

impl Parse for Object {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let mut props = HashMap::new();

        // Loop through all properties
        loop {
            let token = parser.advance()?;
            match token.kind {
                TokenKind::String(name) => {
                    parser.consume(TokenKind::Colon)?;

                    let value = Any::parse(parser)?;
                    props.insert(name, value);

                    // Once no comma at end, we have reached end of object
                    if parser.check(TokenKind::Comma)? {
                        parser.advance()?;
                    } else {
                        break;
                    }
                }
                _ => return Err(parser.make_err_prev(ParserErrKind::UnexpectedToken)),
            }
        }

        parser.consume(TokenKind::RCurlyBracket)?;

        Ok(Self { props })
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    elems: Vec<Any>,
}

impl Parse for Array {
    fn parse(parser: &mut Parser) -> Result<Self, ParserErr> {
        let mut elems = Vec::new();

        // Loop through all elements
        loop {
            let elem = Any::parse(parser)?;
            elems.push(elem);

            // Once no comma at end, we have reached end of array
            if parser.check(TokenKind::Comma)? {
                parser.advance()?;
            } else {
                break;
            }
        }

        parser.consume(TokenKind::RBracket)?;

        Ok(Self { elems })
    }
}
