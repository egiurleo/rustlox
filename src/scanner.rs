use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive)]
pub enum TokenType {
    // Single-character tokens
    LeftParen = 0,
    RightParen = 1,
    LeftBrace = 2,
    RightBrace = 3,
    Comma = 4,
    Dot = 5,
    Minus = 6,
    Plus = 7,
    Semicolon = 8,
    Slash = 9,
    Star = 10,
    // One or two character tokens
    Bang = 11,
    BangEqual = 12,
    Equal = 13,
    EqualEqual = 14,
    Greater = 15,
    GreaterEqual = 16,
    Less = 17,
    LessEqual = 18,
    //Literals
    Identifier = 19,
    String = 20,
    Number = 21,
    //Keywords
    And = 22,
    Class = 23,
    Else = 24,
    False = 25,
    For = 26,
    Fun = 27,
    If = 28,
    Nil = 29,
    Or = 30,
    Print = 31,
    Return = 32,
    Super = 33,
    This = 34,
    True = 35,
    Var = 36,
    While = 37,
    EOF = 38,
}

pub struct Token {
    token_type: TokenType,
    start: usize,
    length: usize,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, length: usize, line: usize) -> Self {
        Token {
            token_type,
            start,
            length,
            line,
        }
    }
}

pub enum ScanError {
    UnexpectedChar { line: usize },
    UnterminatedString { line: usize },
}

#[derive(Default)]
pub struct Scanner {
    line: usize,
    start: usize,
    current: usize,
    source: Vec<u8>,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        Scanner {
            source: source.as_bytes().to_vec(),
            line: 1,
            ..Default::default()
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.skip_whitespace();

        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let c = self.advance();

        if self.is_digit(c) {
            return self.number();
        }

        match c {
            b'(' => return self.make_token(TokenType::LeftParen),
            b')' => return self.make_token(TokenType::RightParen),
            b'{' => return self.make_token(TokenType::LeftBrace),
            b'}' => return self.make_token(TokenType::RightBrace),
            b';' => return self.make_token(TokenType::Semicolon),
            b',' => return self.make_token(TokenType::Comma),
            b'.' => return self.make_token(TokenType::Dot),
            b'-' => return self.make_token(TokenType::Minus),
            b'+' => return self.make_token(TokenType::Plus),
            b'/' => return self.make_token(TokenType::Slash),
            b'*' => return self.make_token(TokenType::Star),
            b'!' => {
                let token_type = if self.matches(b'=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(token_type)
            }
            b'=' => {
                let token_type = if self.matches(b'=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(token_type)
            }
            b'<' => {
                let token_type = if self.matches(b'=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(token_type)
            }
            b'>' => {
                let token_type = if self.matches(b'=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(token_type)
            }
            b'"' => return self.string(),
            _ => Err(ScanError::UnexpectedChar { line: self.line }),
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        self.source[self.current] == b'\0'
    }

    fn make_token(&mut self, token_type: TokenType) -> Result<Token, ScanError> {
        Ok(Token::new(
            token_type,
            self.start,
            self.current - self.start,
            self.line,
        ))
    }

    fn string(&mut self) -> Result<Token, ScanError> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(ScanError::UnterminatedString { line: (self.line) });
        }

        self.advance();

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Result<Token, ScanError> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && self.is_digit(self.peek_next()) {
            self.advance();

            while (self.is_digit(self.peek())) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();

            match c {
                b' ' => {
                    self.advance();
                }
                b'\r' => {
                    self.advance();
                }
                b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn peek(&self) -> u8 {
        self.source[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current + 1]
    }

    fn is_digit(&self, c: u8) -> bool {
        c >= b'0' && c <= b'9'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_scanner_test() {
        let source = "Hello, world!".to_string();
        let scanner = Scanner::new(&source);

        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 0);
        assert_eq!(scanner.source, source.as_bytes());
    }
}
