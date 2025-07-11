use crate::token::{Token, TokenKind};

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

    fn make_token(&mut self, kind: TokenKind<'a>) -> Token {
        let start = self.token_start;
        self.token_start = self.current;

        Token::init(kind, self.line, &self.source[start..self.current])
    }

    fn advance(&mut self) -> char {
        // When advancing, make sure to advance the correct number of bytes
        // A character such as an emoji may be more than 1 byte, so increase `current` by the number
        // of bytes of the char we advanced past
        let remaining = &self.source[self.current..];
        let mut chars = remaining.char_indices();
        let (_, c) = chars.next().expect("Unexpected end of input");
        let (next_byte_index, _) = chars.next().unwrap_or((remaining.len(), ' '));

        self.current += next_byte_index;
        c
    }

    fn peek_unchecked(&self) -> char {
        self.source[self.current..]
            .chars()
            .next()
            .expect("Reached end of source code")
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn peek_prev_unchecked(&self) -> char {
        self.source[self.current - 1..]
            .chars()
            .next()
            .expect("Failed to peek previous character")
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
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
        if self.peek_unchecked() == c {
            self.advance();
            return true;
        }

        false
    }

    fn number(&mut self) -> Token {
        let mut is_float = false;

        // Consume digits - we already know we've got an initial one
        while matches!(self.peek(), Some(c) if c.is_digit(10)) {
            self.advance();
        }

        // If reach a `.`, include it and continue matching digits
        // We know it is a float at this point
        if self.matches('.') {
            is_float = true;
            while matches!(self.peek(), Some(c) if c.is_digit(10)) {
                self.advance();
            }
        }

        let next_char = self.peek();

        // Allow scientific notation e.g. 10e5
        if let Some(c) = next_char {
            if c == 'e' {
                // If using scientific notation, it is a float
                is_float = true;

                self.advance();
                self.matches('-'); // Consume `-` if it exists
                while matches!(self.peek(), Some(c) if c.is_digit(10)) {
                    self.advance();
                }
            } else if c.is_alphabetic() {
                panic!(
                    "Unexpected character in number: '{}{}'",
                    &self.source[self.token_start..self.current],
                    next_char.unwrap(),
                );
            }
        }

        let token_kind = if is_float {
            TokenKind::Float(
                self.source[self.token_start..self.current]
                    .parse()
                    .expect("Failed to parse float."),
            )
        } else {
            TokenKind::Int(
                self.source[self.token_start..self.current]
                    .parse()
                    .expect("Failed to parse int."),
            )
        };

        let token = self.make_token(token_kind);

        return token;
    }

    fn string(&mut self) -> Token {
        while !(self.peek_unchecked() == '"' && self.peek_prev_unchecked() != '\\')
            && !self.is_at_end()
        {
            // Remember to increase line number if multiline string
            if self.peek_unchecked() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        // TODO: handle properly
        if self.is_at_end() {
            panic!("Unterminated string");
        }

        self.advance();
        return self.make_token(TokenKind::String(
            &self.source[self.token_start + 1..self.current - 1],
        ));
    }

    fn keyword(&mut self) -> Token {
        while matches!(self.peek(), Some(c) if c.is_alphabetic()) {
            self.advance();
        }

        let keyword = &self.source[self.token_start..self.current];
        match keyword {
            "true" => self.make_token(TokenKind::Boolean(true)),
            "false" => self.make_token(TokenKind::Boolean(false)),
            "null" => self.make_token(TokenKind::Null),
            _ => panic!("Unrecognised keyword '{keyword}'"),
        }
    }

    fn symbol(&mut self) -> Token {
        let char = self.peek_prev_unchecked();
        match char {
            '{' => self.make_token(TokenKind::LCurlyBracket),
            '}' => self.make_token(TokenKind::RCurlyBracket),
            '[' => self.make_token(TokenKind::LBracket),
            ']' => self.make_token(TokenKind::RBracket),
            ':' => self.make_token(TokenKind::Colon),
            ',' => self.make_token(TokenKind::Comma),
            _ => panic!("Unrecognised symbol '{char}'"),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_at_end() {
            return self.make_token(TokenKind::EOF);
        }

        self.token_start = self.current;

        let c = self.advance();

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
