use crate::chunk::Chunk;
use crate::debug::disassemble_chunk;
use crate::scanner::{Token, TokenType};
use std::io::Write;

pub struct Parser<'a, W: Write> {
    pub current: Option<Token>,
    pub previous: Option<Token>,
    pub had_error: bool,
    writer: &'a mut W,
    panic_mode: bool,
    source: &'a Vec<u8>,
}

impl<'a, W: Write> Parser<'a, W> {
    pub fn new(source: &'a Vec<u8>, writer: &'a mut W) -> Self {
        Parser {
            current: None,
            previous: None,
            writer,
            had_error: false,
            panic_mode: false,
            source,
        }
    }

    pub fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.unwrap(), message);
    }

    pub fn error(&mut self, message: &str) {
        self.error_at(&self.previous.unwrap(), message);
    }

    pub fn previous(&mut self) -> Token {
        self.previous.unwrap()
    }

    pub fn current(&mut self) -> Token {
        self.current.unwrap()
    }

    pub fn disassemble_chunk(&mut self, current_chunk: &Chunk) {
        if !self.had_error {
            disassemble_chunk(current_chunk, "code", self.writer);
        }
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;

        write!(&mut self.writer, "[line {}] Error", token.line).unwrap();

        if token.token_type == TokenType::Eof {
            write!(&mut self.writer, " at end").unwrap();
        } else {
            let lexeme =
                std::str::from_utf8(&self.source[token.start..token.start + token.length]).unwrap();
            write!(&mut self.writer, " at '{}'", lexeme).unwrap();
        }

        writeln!(&mut self.writer, ": {}", message).unwrap();
        self.had_error = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_new_test() {
        let mut output = Vec::new();
        let source = Vec::new();
        let parser = Parser::new(&source, &mut output);

        assert!(parser.current.is_none());
        assert!(parser.previous.is_none());
        assert!(!parser.had_error);
    }

    #[test]
    fn current_returns_token_when_set() {
        let mut output = Vec::new();
        let source = Vec::new();
        let mut parser = Parser::new(&source, &mut output);

        let token = Token::new(TokenType::Plus, 0, 1, 1);
        parser.current = Some(token);

        let result = parser.current();
        assert_eq!(result.token_type, TokenType::Plus);
        assert_eq!(result.start, 0);
        assert_eq!(result.length, 1);
        assert_eq!(result.line, 1);
    }

    #[test]
    fn previous_returns_token_when_set() {
        let mut output = Vec::new();
        let source = Vec::new();
        let mut parser = Parser::new(&source, &mut output);

        let token = Token::new(TokenType::Minus, 5, 1, 2);
        parser.previous = Some(token);

        let result = parser.previous();
        assert_eq!(result.token_type, TokenType::Minus);
        assert_eq!(result.start, 5);
        assert_eq!(result.length, 1);
        assert_eq!(result.line, 2);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn current_panics_when_none() {
        let mut output = Vec::new();
        let source = Vec::new();
        let mut parser = Parser::new(&source, &mut output);

        // current is None by default
        parser.current(); // Should panic
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn previous_panics_when_none() {
        let mut output = Vec::new();
        let source = Vec::new();
        let mut parser = Parser::new(&source, &mut output);

        // previous is None by default
        parser.previous(); // Should panic
    }

    #[test]
    fn error_at_current_writes_error_message() {
        let mut output = Vec::new();
        let source = "+".as_bytes().to_vec();
        let mut parser = Parser::new(&source, &mut output);

        let token = Token::new(TokenType::Plus, 0, 1, 5);
        parser.current = Some(token);

        parser.error_at_current("Unexpected token");
        assert!(parser.had_error);

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "[line 5] Error at '+': Unexpected token\n");
    }
}
