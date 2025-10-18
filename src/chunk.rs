use crate::value::{init_value_array, ValueArray, Value};

#[repr(u8)]
pub enum OpCode {
  OpConstant = 0,
  OpReturn = 1,
}

pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: ValueArray,
  pub lines: Vec<usize>,
}

pub fn init_chunk() -> Chunk {
  Chunk {
    code: Vec::new(),
    constants: init_value_array(),
    lines: Vec::new(),
  }
}

impl Chunk {
  pub fn write(&mut self, byte: u8, line: usize) {
    self.code.push(byte);
    self.lines.push(line);
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self.constants.write(value);
    self.constants.len() - 1
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_chunk_test() {
    let chunk = init_chunk();
    assert_eq!(chunk.code.len(), 0);
    assert_eq!(chunk.constants.len(), 0);
    assert_eq!(chunk.lines.len(), 0);
  }

  #[test]
  fn write_test() {
    let mut chunk = init_chunk();
    chunk.write(OpCode::OpReturn as u8, 123);
    chunk.write(OpCode::OpConstant as u8, 124);

    assert_eq!(chunk.code.len(), 2);
    assert_eq!(chunk.lines.len(), 2);

    assert_eq!(chunk.code[0], OpCode::OpReturn as u8);
    assert_eq!(chunk.lines[0], 123);

    assert_eq!(chunk.code[1], OpCode::OpConstant as u8);
    assert_eq!(chunk.lines[1], 124);
  }

  #[test]
  fn add_constant_test() {
    let mut chunk = init_chunk();
    let result = chunk.add_constant(4.3);

    assert_eq!(result, 0);
    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants.at(0), 4.3);
  }
}
