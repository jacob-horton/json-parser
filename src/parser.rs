use std::collections::HashMap;

use crate::{
    scanner::{Scanner, ScannerErrKind},
    token::{Token, TokenKind},
};

static BUG_PREV_BEFORE_ADVANCE: &'static str =
    "[BUG] called `prev` before advancing - no previous value";

trait Parse {
    fn parse(parser: &mut Parser) -> Self;
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    scanner: Scanner<'a>,

    prev: Option<Token>,
    current: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn parse(source: &str) -> Any {
        let mut scanner = Scanner::init(source);
        // TODO: handle properly
        let current = scanner.next_token().unwrap();

        let mut parser = Parser {
            scanner,
            current: Some(current),
            prev: None,
        };

        let result = Any::parse(&mut parser);
        if parser.current.is_some() {
            panic!("End of file expected");
        }

        return result;
    }

    fn consume(&mut self, kind: TokenKind) {
        if self.check(kind) {
            self.advance();
            return;
        }

        panic!("Expected token");
    }

    fn check(&self, kind: TokenKind) -> bool {
        let curr = self.current.clone().unwrap();
        return curr.kind == kind;
    }

    fn advance(&mut self) -> Token {
        self.prev = self.current.clone();

        // TODO: handle properly
        match self.scanner.next_token() {
            Ok(token) => self.current = Some(token),
            Err(err) => match err.kind {
                ScannerErrKind::EndOfSource => self.current = None,
                _ => panic!("{err:?}"),
            },
        }

        return self.previous();
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
    fn parse(parser: &mut Parser) -> Self {
        let token = parser.advance();
        match token.kind {
            TokenKind::LCurlyBracket => return Self::Object(Object::parse(parser)),
            TokenKind::LBracket => return Self::Array(Array::parse(parser)),
            TokenKind::String(x) => return Self::String(x),
            TokenKind::Float(x) => return Self::Float(x),
            TokenKind::Int(x) => return Self::Int(x),
            TokenKind::Boolean(x) => return Self::Boolean(x),
            TokenKind::Null => return Self::Null,
            _ => panic!("Unexpected token: {:?}", token),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    props: HashMap<String, Any>,
}

impl Parse for Object {
    fn parse(parser: &mut Parser) -> Self {
        let mut props = HashMap::new();

        // Loop through all properties
        loop {
            let token = parser.advance();
            match token.kind {
                TokenKind::String(name) => {
                    parser.consume(TokenKind::Colon);

                    let value = Any::parse(parser);
                    props.insert(name, value);

                    // Once no comma at end, we have reached end of object
                    if parser.check(TokenKind::Comma) {
                        parser.advance();
                    } else {
                        break;
                    }
                }
                _ => panic!("Unexpected token: {:?}", token),
            }
        }

        parser.consume(TokenKind::RCurlyBracket);

        Self { props }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    elems: Vec<Any>,
}

impl Parse for Array {
    fn parse(parser: &mut Parser) -> Self {
        let mut elems = Vec::new();

        // Loop through all elements
        loop {
            let elem = Any::parse(parser);
            elems.push(elem);

            // Once no comma at end, we have reached end of array
            if parser.check(TokenKind::Comma) {
                parser.advance();
            } else {
                break;
            }
        }

        parser.consume(TokenKind::RBracket);

        Self { elems }
    }
}
