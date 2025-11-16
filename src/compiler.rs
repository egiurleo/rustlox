use crate::chunk::{Chunk, OpCode};
use crate::scanner::{ScanError, Scanner, Token, TokenType};
use crate::value::Value;
use num_enum::TryFromPrimitive;
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
        if self.parser.current().token_type == token_type {
            self.advance();
            return;
        }

        self.parser.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.parser.previous().line;
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
        let lexeme = self.scanner.get_lexeme(&self.parser.previous());
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
        let operator_type = self.parser.previous().token_type;

        self.parse_precedence(Precedence::Unary);

        if operator_type == TokenType::Minus {
            self.emit_byte(OpCode::Negate as u8)
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous().token_type;
        let parse_rule = self.get_rule(operator_type);

        let precedence =
            Precedence::try_from(parse_rule.precedence as u8 + 1).expect("Invalid precedence");

        self.parse_precedence(precedence);

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add as u8),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenType::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::Divide as u8),
            _ => (),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let previous = self.parser.previous();
        let prefix_rule = self.get_rule(previous.token_type).prefix;

        // TODO: refactor
        if prefix_rule.is_none() {
            self.parser.error("Expect expression.");
            return;
        }

        prefix_rule.unwrap()(self);

        loop {
            let current_type = self.parser.current().token_type;
            if precedence > self.get_rule(current_type).precedence {
                break;
            }

            self.advance();
            let previous_type = self.parser.previous().token_type;
            let infix_rule = self.get_rule(previous_type).infix;

            infix_rule.unwrap()(self);
        }
    }

    fn get_rule(&self, token_type: TokenType) -> ParseRule<'a, W> {
        match token_type {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Compiler::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Compiler::unary),
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Compiler::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Compiler::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Equality,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::binary),
                precedence: Precedence::Comparison,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
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
}

#[repr(u8)]
#[derive(TryFromPrimitive, std::cmp::PartialOrd, std::cmp::PartialEq)]
enum Precedence {
    None = 0,
    Assignment = 1,
    Or = 2,
    And = 3,
    Equality = 4,
    Comparison = 5,
    Term = 6,
    Factor = 7,
    Unary = 8,
    Call = 9,
    Primary = 10,
}

type ParseFn<'a, W: Write> = fn(&mut Compiler<'a, W>);

struct ParseRule<'a, W: Write> {
    prefix: Option<ParseFn<'a, W>>,
    infix: Option<ParseFn<'a, W>>,
    precedence: Precedence,
}
