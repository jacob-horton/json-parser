#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn init(kind: TokenKind<'a>, line: usize, lexeme: &'a str) -> Self {
        Self { kind, line, lexeme }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    LCurlyBracket,
    RCurlyBracket,

    LBracket,
    RBracket,

    Colon,
    Comma,

    String(&'a str),
    Float(f64),
    Int(i64),
    Boolean(bool),
    Null,
}
