use crate::token::{Token, TokenKind};

static BUG_END_OF_SOURCE: &'static str = "[BUG] Reached end of source when shouldn't be possible";
static BUG_FAILED_PARSE_NUMBER: &'static str =
    "[BUG] Failed to parse number when already validated";
static BUG_PREV_BEFORE_ADVANCE: &'static str =
    "[BUG] Called `prev` before advancing - no previous value";

#[derive(Debug, Clone, PartialEq)]
pub struct ScannerErr {
    pub kind: ScannerErrKind,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScannerErrKind {
    UnexpectedEndOfSource,
    UnterminatedString,
    UnrecognisedSymbol,
    UnrecognisedKeyword,
    InvalidNumber,
    InvalidEscapeSequence,
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

    fn matches_any(&mut self, chars: &[char]) -> bool {
        // If not end of source and character matches, return true
        for c in chars {
            if matches!(self.peek(), Ok(chr) if chr == *c) {
                self.advance().expect(BUG_END_OF_SOURCE);
                return true;
            }
        }

        false
    }

    fn number(&mut self) -> Result<Token, ScannerErr> {
        // Consume digits - we already know we've got an initial one
        while matches!(self.peek(), Ok(c) if c.is_digit(10)) {
            self.advance().expect(BUG_END_OF_SOURCE);
        }

        // If reach a `.`, include it and continue matching digits
        // We know it is a float at this point
        if self.matches('.') {
            while matches!(self.peek(), Ok(c) if c.is_digit(10)) {
                self.advance().expect(BUG_END_OF_SOURCE);
            }
        }

        let next_char = self.peek();

        // Allow scientific notation e.g. 10e5
        if let Ok(c) = next_char {
            if c == 'e' || c == 'E' {
                let mut has_number_after_e = false;

                self.advance().expect(BUG_END_OF_SOURCE);

                // Consume `-` or `+` if it exists
                self.matches_any(&['-', '+']);

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

        let lexeme = &self.source[self.token_start..self.current];
        if lexeme == "-" {
            return Err(self.make_err(ScannerErrKind::InvalidNumber));
        }

        Ok(self.make_token(TokenKind::Number))
    }

    fn is_end_of_string(&self) -> Result<bool, ScannerErr> {
        let next = self.peek()?;

        // Not on a quote, so not end of string
        if next != '"' {
            return Ok(false);
        }

        return Ok(true);
    }

    fn string(&mut self) -> Result<Token, ScannerErr> {
        let mut str_val = String::new();
        while !self.is_end_of_string()? {
            let chr = self.advance().expect(BUG_END_OF_SOURCE);
            if chr == '\n' {
                return Err(self.make_err(ScannerErrKind::UnterminatedString));
            }

            if chr == '\\' {
                let value = match self.advance()? {
                    '"' => '"',
                    '/' => '/',
                    'b' => '\x08',
                    'f' => '\x0C',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    'u' => {
                        // Unicode character - read next 4 hex values and parse
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            hex.push(self.advance()?);
                        }

                        // Covnert hex string to unicode char
                        let digit = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.make_err(ScannerErrKind::InvalidEscapeSequence))?;
                        let unicode = char::from_u32(digit)
                            .ok_or(self.make_err(ScannerErrKind::InvalidEscapeSequence))?;

                        unicode
                    }
                    _ => return Err(self.make_err(ScannerErrKind::InvalidEscapeSequence)),
                };

                str_val.push(value);
                continue;
            }

            str_val.push(chr);
        }

        self.advance().expect(BUG_END_OF_SOURCE);
        return Ok(self.make_token(TokenKind::String(str_val)));
    }

    fn keyword(&mut self) -> Result<Token, ScannerErr> {
        while matches!(self.peek(), Ok(c) if c.is_alphabetic()) {
            self.advance().expect(BUG_END_OF_SOURCE);
        }

        let keyword = &self.source[self.token_start..self.current];
        let kind = match keyword {
            "null" => TokenKind::Null,
            "true" | "false" => TokenKind::Bool,
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

    pub fn next_token(&mut self) -> Result<Option<Token>, ScannerErr> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        self.token_start = self.current;

        let c = self.advance()?;

        if c.is_digit(10) || c == '-' {
            return self.number().map(|x| Some(x));
        }

        if c.is_alphabetic() {
            return self.keyword().map(|x| Some(x));
        }

        if c == '"' {
            return self.string().map(|x| Some(x));
        }

        return self.symbol().map(|x| Some(x));
    }
}

// TODO: check lexemes, not just kinds
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_individual_tokens() {
        let cases = vec![
            ("[", TokenKind::LBracket),
            ("]", TokenKind::RBracket),
            ("{", TokenKind::LCurlyBracket),
            ("}", TokenKind::RCurlyBracket),
            (":", TokenKind::Colon),
            (",", TokenKind::Comma),
            ("1234", TokenKind::Number),
            ("1234e5", TokenKind::Number),
            ("1234E5", TokenKind::Number),
            ("1234.567", TokenKind::Number),
            ("1234.567e5", TokenKind::Number),
            ("1234.567e+5", TokenKind::Number),
            ("1234.567e-5", TokenKind::Number),
            ("\"str a_b\"", TokenKind::String("str a_b".to_string())),
            ("true", TokenKind::Bool),
            ("false", TokenKind::Bool),
            ("null", TokenKind::Null),
        ];

        for (source, expected) in cases {
            let mut scanner = Scanner::init(source);
            assert_eq!(
                Ok(Some(expected)),
                scanner.next_token().map(|x| x.map(|y| y.kind))
            );
        }
    }

    #[test]
    fn test_multiple_tokens() {
        let mut scanner = Scanner::init("{ 1234 12.34 \"hi\" true false null [] }");
        let expected = vec![
            TokenKind::LCurlyBracket,
            TokenKind::Number,
            TokenKind::Number,
            TokenKind::String("hi".to_string()),
            TokenKind::Bool,
            TokenKind::Bool,
            TokenKind::Null,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::RCurlyBracket,
        ];

        for token in expected {
            assert_eq!(
                Ok(Some(token)),
                scanner.next_token().map(|x| x.map(|y| y.kind))
            );
        }
    }

    #[test]
    fn test_whitespace() {
        let mut scanner =
            Scanner::init("{\t\n1234 12.34 \"hi\"\n   \t  \n true \r\n false \rnull [] }");
        let expected = vec![
            TokenKind::LCurlyBracket,
            TokenKind::Number,
            TokenKind::Number,
            TokenKind::String("hi".to_string()),
            TokenKind::Bool,
            TokenKind::Bool,
            TokenKind::Null,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::RCurlyBracket,
        ];

        for token in expected {
            assert_eq!(
                Ok(Some(token)),
                scanner.next_token().map(|x| x.map(|y| y.kind))
            );
        }
    }

    #[test]
    fn test_invalid_tokens() {
        let cases = vec![
            ("\"unterminated\n", ScannerErrKind::UnterminatedString),
            ("\"end of source", ScannerErrKind::UnexpectedEndOfSource),
            ("1234e", ScannerErrKind::InvalidNumber),
            ("1234a", ScannerErrKind::InvalidNumber),
            ("notkeyword", ScannerErrKind::UnrecognisedKeyword),
            ("_", ScannerErrKind::UnrecognisedSymbol),
            ("^", ScannerErrKind::UnrecognisedSymbol),
        ];

        for (source, expected) in cases {
            let mut scanner = Scanner::init(source);
            assert_eq!(Err(expected), scanner.next_token().map_err(|x| x.kind));
        }
    }

    #[test]
    fn test_valid_escape_sequences() {
        let cases = vec![
            (r#""\u00A9""#, "©"),
            (r#""\n""#, "\n"),
            (r#""\r""#, "\r"),
            (r#""\b""#, "\x08"),
            (r#""\/""#, "/"),
            (r#""\\""#, "\\"),
        ];

        for (source, expected) in cases {
            let mut scanner = Scanner::init(source);
            let token = scanner.next_token();

            assert!(matches!(
                token,
                Ok(Some(Token { kind: TokenKind::String(ref s), .. })) if s == expected
            ));
        }
    }

    #[test]
    fn test_invalid_escape_sequences() {
        let cases = vec![
            (r#""\uZZZZ""#, ScannerErrKind::InvalidEscapeSequence),
            (r#""\uD800""#, ScannerErrKind::InvalidEscapeSequence),
            (r#""bad\escape""#, ScannerErrKind::InvalidEscapeSequence),
        ];

        for (source, expected) in cases {
            let mut scanner = Scanner::init(source);
            assert_eq!(Err(expected), scanner.next_token().map_err(|x| x.kind));
        }
    }

    #[test]
    fn test_next_token_at_end() {
        let mut scanner = Scanner::init("\"one_token\"");
        assert!(matches!(scanner.next_token(), Ok(Some(_))));
        assert!(matches!(scanner.next_token(), Ok(None)));
    }
}
