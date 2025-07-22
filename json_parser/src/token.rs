#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub lexeme: String,
}

impl Token {
    pub fn init(kind: TokenKind, line: usize, lexeme: &str) -> Self {
        Self {
            kind,
            line,
            lexeme: lexeme.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LCurlyBracket,
    RCurlyBracket,

    LBracket,
    RBracket,

    Colon,
    Comma,

    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
