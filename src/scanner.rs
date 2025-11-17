use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive, PartialEq, Debug)]
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
    // Make EOF 39 to match the book, which has an extra token type
    Eof = 39,
}

#[derive(Copy, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
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

#[derive(Debug)]
pub enum ScanError {
    UnexpectedChar {},
    UnterminatedString {},
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

    pub fn get_lexeme(&self, token: &Token) -> &str {
        std::str::from_utf8(&self.source[token.start..token.start + token.length]).unwrap()
    }

    pub fn scan_token(&mut self) -> Result<Token, ScanError> {
        self.skip_whitespace();

        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if is_alpha(c) {
            return self.identifier();
        }

        if is_digit(c) {
            return self.number();
        }

        match c {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b';' => self.make_token(TokenType::Semicolon),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b'/' => self.make_token(TokenType::Slash),
            b'*' => self.make_token(TokenType::Star),
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
            b'"' => self.string(),
            _ => Err(ScanError::UnexpectedChar {}),
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
        self.current >= self.source.len()
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
            return Err(ScanError::UnterminatedString {});
        }

        self.advance();

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Result<Token, ScanError> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Result<Token, ScanError> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.source[self.start] {
            b'a' => self.check_keyword(1, 2, b"nd", TokenType::And),
            b'c' => self.check_keyword(1, 4, b"lass", TokenType::Class),
            b'e' => self.check_keyword(1, 3, b"lse", TokenType::Else),
            b'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        b'a' => self.check_keyword(2, 3, b"lse", TokenType::False),
                        b'o' => self.check_keyword(2, 1, b"r", TokenType::For),
                        b'u' => self.check_keyword(2, 1, b"n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'i' => self.check_keyword(1, 1, b"f", TokenType::If),
            b'n' => self.check_keyword(1, 2, b"il", TokenType::Nil),
            b'o' => self.check_keyword(1, 1, b"r", TokenType::Or),
            b'p' => self.check_keyword(1, 4, b"rint", TokenType::Print),
            b'r' => self.check_keyword(1, 5, b"eturn", TokenType::Return),
            b's' => self.check_keyword(1, 4, b"uper", TokenType::Super),
            b't' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        b'h' => self.check_keyword(2, 2, b"is", TokenType::This),
                        b'r' => self.check_keyword(2, 2, b"ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            b'v' => self.check_keyword(1, 2, b"ar", TokenType::Var),
            b'w' => self.check_keyword(1, 4, b"hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(
        &self,
        start: usize,
        length: usize,
        rest: &[u8],
        token_type: TokenType,
    ) -> TokenType {
        if self.current - self.start == start + length
            && &self.source[self.start + start..self.start + start + length] == rest
        {
            return token_type;
        }

        TokenType::Identifier
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
        if self.is_at_end() {
            return b'\0';
        }

        self.source[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current + 1]
    }
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == b'_'
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

    #[test]
    fn scan_basic_token_test() {
        let source = "(){};,.-+/*! != = == < <= > >=".to_string();
        let mut scanner = Scanner::new(&source);

        let mut token: Token;

        let token_types = [
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Semicolon,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Slash,
            TokenType::Star,
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Eof,
        ];

        for token_type in token_types {
            token = scanner.scan_token().unwrap();
            assert_eq!(token.token_type, token_type);
        }
    }

    #[test]
    fn scan_identifier_test() {
        let source = "apple and crazy class elephant else faint false for fun ice if nope nil oops or pretty print rope return sit super tiny this true vapid var wart while".to_string();
        let mut scanner = Scanner::new(&source);

        let mut token: Token;

        let token_types = [
            TokenType::Identifier,
            TokenType::And,
            TokenType::Identifier,
            TokenType::Class,
            TokenType::Identifier,
            TokenType::Else,
            TokenType::Identifier,
            TokenType::False,
            TokenType::For,
            TokenType::Fun,
            TokenType::Identifier,
            TokenType::If,
            TokenType::Identifier,
            TokenType::Nil,
            TokenType::Identifier,
            TokenType::Or,
            TokenType::Identifier,
            TokenType::Print,
            TokenType::Identifier,
            TokenType::Return,
            TokenType::Identifier,
            TokenType::Super,
            TokenType::Identifier,
            TokenType::This,
            TokenType::True,
            TokenType::Identifier,
            TokenType::Var,
            TokenType::Identifier,
            TokenType::While,
        ];

        for token_type in token_types {
            token = scanner.scan_token().unwrap();
            assert_eq!(token.token_type, token_type);
        }
    }

    #[test]
    fn scan_number_test() {
        let source = "1 5.0 300 305.2".to_string();
        let mut scanner = Scanner::new(&source);
        let mut token: Token;

        let token_types = [
            TokenType::Number,
            TokenType::Number,
            TokenType::Number,
            TokenType::Number,
        ];

        for token_type in token_types {
            token = scanner.scan_token().unwrap();
            assert_eq!(token.token_type, token_type);
        }
    }

    #[test]
    fn scan_string_test() {
        let source = "\"Hello, world!\"".to_string();
        let mut scanner = Scanner::new(&source);

        let token = scanner.scan_token().unwrap();
        assert_eq!(token.token_type, TokenType::String);
    }

    #[test]
    fn scan_unterminated_string_test() {
        let source = "\"Hello, world!".to_string();
        let mut scanner = Scanner::new(&source);

        let result = scanner.scan_token();
        assert!(matches!(result, Err(ScanError::UnterminatedString {})));
    }

    #[test]
    fn scan_unexpected_char() {
        let source = "#".to_string();
        let mut scanner = Scanner::new(&source);

        let result = scanner.scan_token();
        assert!(matches!(result, Err(ScanError::UnexpectedChar {})));
    }
}
