use crate::chunk::{Chunk, OpCode};
use std::io::Write;

pub fn _disassemble_chunk<W: Write>(
  chunk: &Chunk,
  name: &str,
  writer: &mut W,
) {
  writeln!(writer, "== {} ==", name).unwrap();

  let mut offset = 0;
  while offset < chunk.code.len() {
    offset = disassemble_instruction(chunk, offset, writer);
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

  let instruction = *chunk.code.get(offset).expect("Index out of bounds");

  match OpCode::try_from(instruction) {
    Ok(OpCode::Constant) => constant_instruction("OP_CONSTANT", chunk, offset, writer),
    Ok(OpCode::Add) => simple_instruction("OP_ADD", offset, writer),
    Ok(OpCode::Subtract) => simple_instruction("OP_SUBTRACT", offset, writer),
    Ok(OpCode::Multiply) => simple_instruction("OP_MULTIPLY", offset, writer),
    Ok(OpCode::Divide) => simple_instruction("OP_DIVIDE", offset, writer),
    Ok(OpCode::Negate) => simple_instruction("OP_NEGATE", offset, writer),
    Ok(OpCode::Return) => simple_instruction("OP_RETURN", offset, writer),
    Err(_) => {
      writeln!(writer, "Unknown opcode: {:?}", instruction).unwrap();
      offset + 1
    }
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
    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_constant_test() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_negate_test() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::Negate as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_NEGATE\n\
    0003    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_add_test() {
    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(5.3);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Add as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_CONSTANT         1 '5.3'\n\
    0004    | OP_ADD\n\
    0005    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_subtract_test() {
    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(5.3);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Subtract as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_CONSTANT         1 '5.3'\n\
    0004    | OP_SUBTRACT\n\
    0005    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_multiply_test() {
    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(5.3);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Multiply as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_CONSTANT         1 '5.3'\n\
    0004    | OP_MULTIPLY\n\
    0005    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }

  #[test]
  fn disassemble_op_divide_test() {
    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    constant = chunk.add_constant(5.3);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Multiply as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    let mut output = Vec::new();
    _disassemble_chunk(&chunk, "test chunk", &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let expectation = "== test chunk ==\n\
    0000  123 OP_CONSTANT         0 '1.2'\n\
    0002    | OP_CONSTANT         1 '5.3'\n\
    0004    | OP_MULTIPLY\n\
    0005    | OP_RETURN\n";

    assert_eq!(output_str, expectation);
  }
}
