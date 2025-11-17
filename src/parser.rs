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

    pub fn error(&mut self, message: &str) {
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
}
