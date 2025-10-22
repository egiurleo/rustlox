use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use crate::debug::disassemble_instruction;
use std::io::Write;

const DEBUG_TRACE: bool =
  option_env!("DEBUG_TRACE_EXECUTION").is_some();

const STACK_MAX: usize = 256;

pub enum InterpretResult {
  InterpretOk = 0,
  // InterpretCompileError = 1,
  // InterpretRuntimeError = 2,
}

pub struct VM {
  chunk: Chunk,
  ip: u8,
  stack: [Value; STACK_MAX],
  stack_top: usize,
}

impl Default for VM {
  fn default() -> Self {
    VM {
      chunk: Chunk::default(),
      ip: 0,
      stack: [0.0; STACK_MAX],
      stack_top: 0,
    }
  }
}

impl VM {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn interpret<W: Write>(&mut self, chunk: Chunk, writer: &mut W) -> InterpretResult {
    self.chunk = chunk;
    self.ip = 0;

    self.run(writer)
  }

  pub fn _reset_stack(&mut self) {
    self.stack_top = 0;
  }

  pub fn push(&mut self, value: Value) {
    self.stack[self.stack_top] = value;
    self.stack_top += 1;
  }

  pub fn pop(&mut self) -> Value {
    self.stack_top -= 1;
    *self.stack.get(self.stack_top).expect("Stack index out of bounds")
  }

  fn run<W: Write>(&mut self, writer: &mut W) -> InterpretResult {
    let mut instruction: u8;

    loop {
      if DEBUG_TRACE {
        write!(writer, "          ").unwrap();
        for i in 0..self.stack_top {
          let value = self.stack.get(i).expect("Stack index out of bounds");
          write!(writer, "[ {} ]", value).unwrap();
        }
        write!(writer, "\n").unwrap();

        disassemble_instruction(&self.chunk, self.ip as usize, writer);
      }

      instruction = self.read_byte();

      match OpCode::try_from(instruction) {
        Ok(OpCode::OpConstant) => {
          let constant = self.read_constant();
          self.push(constant);
        },
        Ok(OpCode::OpAdd) => self.binary_op(|a, b| a + b),
        Ok(OpCode::OpSubtract) => self.binary_op(|a, b| a - b),
        Ok(OpCode::OpMultiply) => self.binary_op(|a, b| a * b),
        Ok(OpCode::OpDivide) => self.binary_op(|a, b| a / b),
        Ok(OpCode::OpNegate) => {
          let pop = self.pop();
          self.push(-1.0 * pop);
        },
        Ok(OpCode::OpReturn) => {
          writeln!(writer, "{}", self.pop()).unwrap();
          return InterpretResult::InterpretOk;
        },
        Err(_) => panic!("Unknown opcode: {}", instruction),
      }
    }
  }


  #[inline]
  fn binary_op<F>(&mut self, op: F)
  where
    F: Fn(Value, Value) -> Value
  {
    let b = self.pop();
    let a = self.pop();
    self.push(op(a, b));
  }

  #[inline]
  fn read_byte(&mut self) -> u8 {
    let byte = *self.chunk.code.get(self.ip as usize).expect("Index is out of bounds");
    self.ip += 1;
    byte
  }

  #[inline]
  fn read_constant(&mut self) -> Value {
    let byte = self.read_byte();
    self.chunk.constants.at(byte as usize)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn interpret_constant_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "1.2\n");
  }

  #[test]
  fn interpret_negation_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::OpNegate as u8, 123);
    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "-1.2\n");
  }

  #[test]
  fn interpret_addition_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(2.3);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpAdd as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "3.5\n");
  }

  #[test]
  fn interpret_subtraction_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let mut constant = chunk.add_constant(1.5);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(0.3);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpSubtract as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "1.2\n");
  }

  #[test]
  fn interpret_multiplication_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(2.0);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpMultiply as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "2.4\n");
  }

  #[test]
  fn interpret_division_test() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();
    let mut constant = chunk.add_constant(2.4);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(2.0);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpDivide as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = vm.interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "1.2\n");
  }
}
