use crate::chunk::Chunk;
use crate::scanner::{ScanError, Scanner, Token, TokenType};
use std::io::Write;

pub struct Compiler<'a, W: Write> {
    parser: Parser<'a, W>,
    scanner: Scanner,
}

impl<'a, W: Write> Compiler<'a, W> {
    pub fn new(source: &String, writer: &'a mut W) -> Self {
        Compiler {
            parser: Parser::new(writer),
            scanner: Scanner::new(source),
        }
    }

    pub fn compile(&mut self, _chunk: &Chunk) -> bool {
        self.advance();
        // self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");

        false
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.parser.current = Some(token);
                }
                Err(err) => match err {
                    ScanError::UnexpectedChar { line: _ } => {
                        self.parser.error_at_current("Unexpected character");
                    }
                    ScanError::UnterminatedString { line: _ } => {
                        self.parser.error_at_current("Unterminated string");
                    }
                },
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.current.unwrap().token_type == token_type {
            self.advance();
            return;
        }

        self.parser.error_at_current(message);
    }
}

struct Parser<'a, W: Write> {
    pub current: Option<Token>,
    pub previous: Option<Token>,
    writer: &'a mut W,
    had_error: bool,
    panic_mode: bool,
}

impl<'a, W: Write> Parser<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Parser {
            current: None,
            previous: None,
            writer,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current.unwrap(), message);
    }

    pub fn _error(&mut self, message: &str) {
        self.error_at(&self.previous.unwrap(), message);
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
            write!(&mut self.writer, " at <something here>").unwrap();
        }

        writeln!(&mut self.writer, ": {}", message).unwrap();
        self.had_error = true;
    }
}
