use crate::chunk::{Chunk, OpCode};
use crate::scanner::{ScanError, Scanner, Token, TokenType};
use crate::value::Value;
use std::io::Write;

pub struct Compiler<'a, W: Write> {
    parser: Parser<'a, W>,
    scanner: Scanner,
    compiling_chunk: Option<&'a mut Chunk>,
}

impl<'a, W: Write> Compiler<'a, W> {
    pub fn new(source: &String, writer: &'a mut W) -> Self {
        Compiler {
            parser: Parser::new(writer),
            scanner: Scanner::new(source),
            compiling_chunk: None,
        }
    }

    pub fn compile(&mut self, chunk: &'a mut Chunk) -> bool {
        self.compiling_chunk = Some(chunk);

        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");

        self.end_compiler();
        !self.parser.had_error
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

    fn emit_byte(&mut self, byte: u8) {
        let line = self.parser.previous.unwrap().line;
        self.current_chunk().write(byte, line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        self.compiling_chunk.as_mut().unwrap()
    }

    fn end_compiler(&mut self) {
        self.emit_return();
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return as u8);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let lexeme = self.scanner.get_lexeme(&self.parser.previous.unwrap());
        let value: f64 = lexeme.parse().expect("Failed to parse number");

        self.emit_constant(value);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);

        if constant > u8::MAX as usize {
            self.parser.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.unwrap().token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            _ => return,
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {}
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
}

enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}
