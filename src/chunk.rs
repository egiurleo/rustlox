use crate::value::{init_value_array, ValueArray, Value};

pub enum OpCode {
  OpConstant = 0,
  OpReturn = 1,
}

pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: ValueArray,
}

pub fn init_chunk() -> Chunk {
  Chunk {
    code: Vec::new(),
    constants: init_value_array(),
  }
}

impl Chunk {
  pub fn write(&mut self, byte: u8) {
    self.code.push(byte);
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
  }

  #[test]
  fn write_test() {
    let mut chunk = init_chunk();
    chunk.write(OpCode::OpReturn as u8);
    chunk.write(OpCode::OpConstant as u8);

    assert_eq!(chunk.code.len(), 2);
    assert_eq!(chunk.code[0], OpCode::OpReturn as u8);
    assert_eq!(chunk.code[1], OpCode::OpConstant as u8);
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
