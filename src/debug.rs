use crate::chunk::{Chunk, OpCode};
use std::io::Write;

pub fn disassemble_chunk<W: Write>(
  chunk: &Chunk,
  name: &str,
  writer: &mut W,
) {
  writeln!(writer, "== {} ==", name).unwrap();

  let mut offset = 0;
  while offset < chunk.code.len() {
    offset = disassemble_instruction(&chunk, offset, writer);
  }
}

pub fn disassemble_instruction<W: Write>(
  chunk: &Chunk,
  offset: usize,
  writer: &mut W,
) -> usize {
  write!(writer, "{:04} ", offset).unwrap();

  if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
    write!(writer, "   | ").unwrap();
  } else {
    write!(writer, " {} ", chunk.lines[offset]).unwrap();
  }

  let instruction = chunk.code[offset];

  if instruction == OpCode::OpConstant as u8 {
    constant_instruction("OP_CONSTANT", chunk, offset, writer)
  } else if instruction == OpCode::OpReturn as u8 {
    simple_instruction("OP_RETURN", offset, writer)
  } else {
    writeln!(writer, "Unknown opcode: {:?}", instruction).unwrap();
    offset + 1
  }
}

fn simple_instruction<W: Write>(name: &str, offset: usize, writer: &mut W) -> usize {
  writeln!(writer, "{}", name).unwrap();
  offset + 1
}

fn constant_instruction<W: Write>(name: &str, chunk: &Chunk, offset: usize, writer: &mut W) -> usize {
  let constant = chunk.code[offset + 1];
  write!(writer, "{}         {} ", name, constant).unwrap();
  write!(writer, "'{}'", chunk.constants.at(constant as usize)).unwrap();
  writeln!(writer).unwrap();
  offset + 2
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn disassemble_op_return_test() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_constant_test() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }
}
