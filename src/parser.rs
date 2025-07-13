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
        let curr = self.peek();
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

    fn peek(&self) -> Token {
        return self.current.clone().unwrap();
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

// Performs rust matching, and advances if a case is matched
macro_rules! match_and_advance {
    ($parser:expr, {
        $($pat:pat => $branch:expr),* $(,)?
    }) => {{
        let kind = $parser.peek().kind;
        match kind {
            $(
                $pat => {
                    $parser.advance();
                    $branch;
                    true
                }
            )*
            _ => false,
        }
    }}
}

impl Parse for Any {
    fn parse(parser: &mut Parser) -> Self {
        match_and_advance!(parser, {
            TokenKind::LCurlyBracket => return Self::Object(Object::parse(parser)),
            TokenKind::LBracket => return Self::Array(Array::parse(parser)),
            TokenKind::String(x) => return Self::String(x),
            TokenKind::Float(x) => return Self::Float(x),
            TokenKind::Int(x) => return Self::Int(x),
            TokenKind::Boolean(x) => return Self::Boolean(x),
            TokenKind::Null => return Self::Null,
        });

        panic!("Unexpected token: {:?}", parser.peek())
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    props: HashMap<String, Any>,
}

impl Parse for Object {
    fn parse(parser: &mut Parser) -> Self {
        let mut props = HashMap::new();

        // TODO: use `loop` and break if reached `}`
        let mut has_another_field = true;
        while has_another_field {
            let success = match_and_advance!(parser, {
                TokenKind::String(name) => {
                    parser.consume(TokenKind::Colon);
                    props.insert(name, Any::parse(parser));

                    has_another_field = match_and_advance!(parser, {TokenKind::Comma => {}});
                }
            });

            if !success {
                panic!("Unexpected token: {:?}", parser.peek());
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

        let mut has_another_field = true;
        while has_another_field {
            elems.push(Any::parse(parser));

            has_another_field = match_and_advance!(parser, {TokenKind::Comma => {}});
        }

        parser.consume(TokenKind::RBracket);

        Self { elems }
    }
}
