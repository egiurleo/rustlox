use crate::chunk::{Chunk, OpCode};
use crate::parser::Parser;
use crate::scanner::{ScanError, Scanner, TokenType};
use crate::value::Value;
use num_enum::TryFromPrimitive;
use std::io::Write;

const DEBUG_PRINT_CODE: bool = option_env!("DEBUG_PRINT_CODE").is_some();

pub struct Compiler<'a, W: Write> {
    parser: Parser<'a, W>,
    scanner: Scanner<'a>,
    compiling_chunk: Option<&'a mut Chunk>,
}

impl<'a, W: Write> Compiler<'a, W> {
    pub fn new(source: &'a Vec<u8>, writer: &'a mut W) -> Self {
        Compiler {
            parser: Parser::new(source, writer),
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
                    break;
                }
                Err(err) => match err {
                    ScanError::UnexpectedChar {} => {
                        self.parser.error_at_current("Unexpected character");
                    }
                    ScanError::UnterminatedString {} => {
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

        if DEBUG_PRINT_CODE {
            let current_chunk = self.compiling_chunk.as_ref().unwrap();
            self.parser.disassemble_chunk(current_chunk);
        }
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

#[repr(u8)]
#[derive(TryFromPrimitive, std::cmp::PartialOrd, std::cmp::PartialEq)]
pub enum Precedence {
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

pub type ParseFn<'a, W> = fn(&mut Compiler<'a, W>);

pub struct ParseRule<'a, W: Write> {
    prefix: Option<ParseFn<'a, W>>,
    infix: Option<ParseFn<'a, W>>,
    precedence: Precedence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_compiler_test() {
        let source = "123".as_bytes().to_vec();
        let mut output = Vec::new();
        let compiler = Compiler::new(&source, &mut output);

        // Verify the compiler is created with expected initial state
        assert!(compiler.compiling_chunk.is_none());
        assert!(!compiler.parser.had_error);
    }

    #[test]
    fn compile_simple_number_test() {
        let source = "42".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result); // Compilation should succeed
        assert!(!compiler.parser.had_error);

        assert_eq!(chunk.code.len(), 3);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[1], 0); // Index of the constant
        assert_eq!(chunk.code[2], OpCode::Return as u8);

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants.at(0), 42.0);
    }

    #[test]
    fn compile_addition_test() {
        let source = "1 + 2".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should have: CONSTANT, 0, CONSTANT, 1, ADD, RETURN
        assert_eq!(chunk.code.len(), 6);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[1], 0); // Index of first constant
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[3], 1); // Index of second constant
        assert_eq!(chunk.code[4], OpCode::Add as u8);
        assert_eq!(chunk.code[5], OpCode::Return as u8);
        assert_eq!(chunk.constants.len(), 2);
        assert_eq!(chunk.constants.at(0), 1.0);
        assert_eq!(chunk.constants.at(1), 2.0);
    }

    #[test]
    fn compile_multiplication_test() {
        let source = "3 * 4".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should have: CONSTANT, 0, CONSTANT, 1, MULTIPLY, RETURN
        assert_eq!(chunk.code.len(), 6);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[1], 0);
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[3], 1);
        assert_eq!(chunk.code[4], OpCode::Multiply as u8);
        assert_eq!(chunk.code[5], OpCode::Return as u8);
        assert_eq!(chunk.constants.at(0), 3.0);
        assert_eq!(chunk.constants.at(1), 4.0);
    }

    #[test]
    fn compile_unary_negation_test() {
        let source = "-5".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should have: CONSTANT, 0, NEGATE, RETURN
        assert_eq!(chunk.code.len(), 4);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[1], 0);
        assert_eq!(chunk.code[2], OpCode::Negate as u8);
        assert_eq!(chunk.code[3], OpCode::Return as u8);
        assert_eq!(chunk.constants.at(0), 5.0);
    }

    #[test]
    fn compile_grouping_test() {
        let source = "(1 + 2)".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should have: CONSTANT, 0, CONSTANT, 1, ADD, RETURN
        assert_eq!(chunk.code.len(), 6);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[4], OpCode::Add as u8);
        assert_eq!(chunk.code[5], OpCode::Return as u8);
    }

    #[test]
    fn compile_precedence_test() {
        let source = "1 + 2 * 3".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should evaluate as 1 + (2 * 3), so: CONSTANT(1), CONSTANT(2), CONSTANT(3), MULTIPLY, ADD, RETURN
        assert_eq!(chunk.code.len(), 9);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[1], 0); // 1
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[3], 1); // 2
        assert_eq!(chunk.code[4], OpCode::Constant as u8);
        assert_eq!(chunk.code[5], 2); // 3
        assert_eq!(chunk.code[6], OpCode::Multiply as u8);
        assert_eq!(chunk.code[7], OpCode::Add as u8);
        assert_eq!(chunk.code[8], OpCode::Return as u8);
        assert_eq!(chunk.constants.at(0), 1.0);
        assert_eq!(chunk.constants.at(1), 2.0);
        assert_eq!(chunk.constants.at(2), 3.0);
    }

    #[test]
    fn compile_subtraction_and_division_test() {
        let source = "10 - 2 / 2".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should evaluate as 10 - (2 / 2): CONSTANT(10), CONSTANT(2), CONSTANT(2), DIVIDE, SUBTRACT, RETURN
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[4], OpCode::Constant as u8);
        assert_eq!(chunk.code[6], OpCode::Divide as u8);
        assert_eq!(chunk.code[7], OpCode::Subtract as u8);
        assert_eq!(chunk.code[8], OpCode::Return as u8);
    }

    #[test]
    fn compile_complex_expression_test() {
        let source = "-(1 + 2) * 3".as_bytes().to_vec();
        let mut output = Vec::new();
        let mut compiler = Compiler::new(&source, &mut output);
        let mut chunk = Chunk::new();

        let result = compiler.compile(&mut chunk);

        assert!(result);
        assert!(!compiler.parser.had_error);
        // Should evaluate as (-(1 + 2)) * 3
        // CONSTANT(1), CONSTANT(2), ADD, NEGATE, CONSTANT(3), MULTIPLY, RETURN
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.code[2], OpCode::Constant as u8);
        assert_eq!(chunk.code[4], OpCode::Add as u8);
        assert_eq!(chunk.code[5], OpCode::Negate as u8);
        assert_eq!(chunk.code[6], OpCode::Constant as u8);
        assert_eq!(chunk.code[8], OpCode::Multiply as u8);
        assert_eq!(chunk.code[9], OpCode::Return as u8);
    }
}
