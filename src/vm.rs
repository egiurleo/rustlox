use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use crate::debug::disassemble_instruction;
use std::io::Write;

const DEBUG_TRACE: bool =
  option_env!("DEBUG_TRACE_EXECUTION").is_some();

pub enum InterpretResult {
  InterpretOk = 0,
  // InterpretCompileError = 1,
  // InterpretRuntimeError = 2,
}

#[derive(Default)]
pub struct VM {
  chunk: Chunk,
  ip: u8,
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

  fn run<W: Write>(&mut self, writer: &mut W) -> InterpretResult {
    let mut instruction: u8;

    loop {
      if DEBUG_TRACE {
        disassemble_instruction(&self.chunk, self.ip as usize, writer);
      }

      instruction = self.read_byte();
      if instruction == OpCode::OpConstant as u8 {
        let constant = self.read_constant();
        writeln!(writer, "{}", constant).unwrap();
      } else if instruction == OpCode::OpReturn as u8 {
        return InterpretResult::InterpretOk;
      }
    }
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
  fn interpret_test() {
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
}
