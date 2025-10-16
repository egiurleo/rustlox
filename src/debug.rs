use crate::chunk::{Chunk, OpCode};
use std::io::{Write};

pub fn disassemble_chunk<W: Write>(
  chunk: Chunk,
  name: &str,
  writer: &mut W,
) {
  writeln!(writer, "== {} ==", name).unwrap();

  let mut offset = 0;
  while offset < chunk.code.len() {
    offset = disassemble_instruction(&chunk, offset, writer);
  }
}

fn disassemble_instruction<W: Write>(
  chunk: &Chunk,
  offset: usize,
  writer: &mut W,
) -> usize {
  write!(writer, "{:04} ", offset).unwrap();
  let instruction = chunk.code[offset];
  if instruction == OpCode::OpReturn as u8 {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn disassemble_op_return_test() {
    let mut chunk = crate::chunk::init_chunk();
    chunk.write(OpCode::OpReturn as u8);

    let mut output = Vec::new();
    disassemble_chunk(chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("== test chunk =="));
    assert!(output_str.contains("0000 OP_RETURN"));
  }
}
