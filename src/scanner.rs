use crate::token::{Token, TokenKind};

static BUG_END_OF_SOURCE: &'static str = "[BUG] Reached end of source when shouldn't be possible";
static BUG_FAILED_PARSE_NUMBER: &'static str =
    "[BUG] Failed to parse number when already validated";
static BUG_PREV_BEFORE_ADVANCE: &'static str =
    "[BUG] called `prev` before advancing - no previous value";

#[derive(Debug, Clone)]
pub struct ScannerErr {
    pub kind: ScannerErrKind,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug, Clone)]
pub enum ScannerErrKind {
    EndOfSource,
    UnexpectedEndOfSource,
    UnterminatedString,
    UnrecognisedSymbol,
    UnrecognisedKeyword,
    InvalidNumber,
}

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    source: &'a str,
    token_start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn init(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            token_start: 0,
            line: 1,
        }
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        let start = self.token_start;
        self.token_start = self.current;

        Token::init(kind, self.line, &self.source[start..self.current])
    }

    fn make_err(&self, kind: ScannerErrKind) -> ScannerErr {
        ScannerErr {
            kind,
            line: self.line,
            lexeme: self.source[self.token_start..self.current].to_string(),
        }
    }

    fn advance(&mut self) -> Result<char, ScannerErr> {
        // When advancing, make sure to advance the correct number of bytes
        // A character such as an emoji may be more than 1 byte, so increase `current` by the number
        // of bytes of the char we advanced past
        let remaining = &self.source[self.current..];
        let mut chars = remaining.char_indices();
        let (_, c) = chars
            .next()
            .ok_or(self.make_err(ScannerErrKind::UnexpectedEndOfSource))?;
        let (next_byte_index, _) = chars.next().unwrap_or((remaining.len(), ' '));

        self.current += next_byte_index;
        Ok(c)
    }

    fn peek(&self) -> Result<char, ScannerErr> {
        self.source[self.current..]
            .chars()
            .next()
            .ok_or(self.make_err(ScannerErrKind::UnexpectedEndOfSource))
    }

    fn prev(&self) -> char {
        self.source[self.current - 1..]
            .chars()
            .next()
            .expect(BUG_PREV_BEFORE_ADVANCE)
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Ok(' ' | '\t' | '\r') => {
                    self.advance().expect(BUG_END_OF_SOURCE);
                }
                Ok('\n') => {
                    self.line += 1;
                    self.advance().expect(BUG_END_OF_SOURCE);
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn matches(&mut self, c: char) -> bool {
        // If not end of source and character matches, return true
        if matches!(self.peek(), Ok(chr) if chr == c) {
            self.advance().expect(BUG_END_OF_SOURCE);
            return true;
        }

        false
    }

    fn number(&mut self) -> Result<Token, ScannerErr> {
        let mut is_float = false;

        // Consume digits - we already know we've got an initial one
        while matches!(self.peek(), Ok(c) if c.is_digit(10)) {
            self.advance().expect(BUG_END_OF_SOURCE);
        }

        // If reach a `.`, include it and continue matching digits
        // We know it is a float at this point
        if self.matches('.') {
            is_float = true;
            while matches!(self.peek(), Ok(c) if c.is_digit(10)) {
                self.advance().expect(BUG_END_OF_SOURCE);
            }
        }

        let next_char = self.peek();

        // Allow scientific notation e.g. 10e5
        if let Ok(c) = next_char {
            if c == 'e' {
                // If using scientific notation, it is a float
                is_float = true;
                let mut has_number_after_e = false;

                self.advance().expect(BUG_END_OF_SOURCE);
                self.matches('-'); // Consume `-` if it exists
                while matches!(self.peek(), Ok(c) if c.is_digit(10)) {
                    self.advance().expect(BUG_END_OF_SOURCE);
                    has_number_after_e = true;
                }

                if !has_number_after_e {
                    return Err(self.make_err(ScannerErrKind::InvalidNumber));
                }
            } else if c.is_alphabetic() {
                return Err(self.make_err(ScannerErrKind::InvalidNumber));
            }
        }

        let token_kind = if is_float {
            let lexeme = &self.source[self.token_start..self.current];
            let literal = lexeme.parse().expect(BUG_FAILED_PARSE_NUMBER);
            TokenKind::Float(literal)
        } else {
            let lexeme = &self.source[self.token_start..self.current];
            let literal = lexeme.parse().expect(BUG_FAILED_PARSE_NUMBER);
            TokenKind::Int(literal)
        };

        Ok(self.make_token(token_kind))
    }

    fn is_end_of_string(&self) -> Result<bool, ScannerErr> {
        let next = self.peek()?;

        // Not on a quote, so not end of string
        if next != '"' {
            return Ok(false);
        }

        let curr = self.prev();

        // Escaping the quote
        if curr == '\\' {
            return Ok(false);
        }

        return Ok(true);
    }

    fn string(&mut self) -> Result<Token, ScannerErr> {
        while !self.is_end_of_string()? {
            if matches!(self.peek(), Ok('\n')) {
                return Err(self.make_err(ScannerErrKind::UnterminatedString));
            }

            self.advance().expect(BUG_END_OF_SOURCE);
        }

        self.advance().expect(BUG_END_OF_SOURCE);
        let unquoted_value = &self.source[self.token_start + 1..self.current - 1];
        return Ok(self.make_token(TokenKind::String(unquoted_value.to_string())));
    }

    fn keyword(&mut self) -> Result<Token, ScannerErr> {
        while matches!(self.peek(), Ok(c) if c.is_alphabetic()) {
            self.advance().expect(BUG_END_OF_SOURCE);
        }

        let keyword = &self.source[self.token_start..self.current];
        let kind = match keyword {
            "null" => TokenKind::Null,
            "true" => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false),
            _ => Err(self.make_err(ScannerErrKind::UnrecognisedKeyword))?,
        };

        Ok(self.make_token(kind))
    }

    fn symbol(&mut self) -> Result<Token, ScannerErr> {
        let char = self.prev();
        let kind = match char {
            '{' => TokenKind::LCurlyBracket,
            '}' => TokenKind::RCurlyBracket,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            _ => Err(self.make_err(ScannerErrKind::UnrecognisedSymbol))?,
        };

        Ok(self.make_token(kind))
    }

    pub fn next_token(&mut self) -> Result<Token, ScannerErr> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Err(self.make_err(ScannerErrKind::EndOfSource));
        }

        self.token_start = self.current;

        let c = self.advance()?;

        if c.is_digit(10) || c == '-' {
            return self.number();
        }

        if c.is_alphabetic() {
            return self.keyword();
        }

        if c == '"' {
            return self.string();
        }

        return self.symbol();
    }
}
