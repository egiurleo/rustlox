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
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        Err(ScanError::UnexpectedChar { line: self.line })
    }

    fn is_at_end(&mut self) -> bool {
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
