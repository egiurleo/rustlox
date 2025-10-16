pub enum OpCode {
  OpReturn = 0,
}

pub struct Chunk {
  pub code: Vec<u8>,
}

pub fn init_chunk() -> Chunk {
  Chunk {
    code: Vec::new(),
  }
}

impl Chunk {
  pub fn write(&mut self, byte: u8) {
    self.code.push(byte);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_chunk_test() {
    let chunk = init_chunk();
    assert_eq!(chunk.code.len(), 0);
  }

  #[test]
  fn write_test() {
    let mut chunk = init_chunk();
    chunk.write(OpCode::OpReturn as u8);
    chunk.write(OpCode::OpReturn as u8);

    assert_eq!(chunk.code.len(), 2);
    assert_eq!(chunk.code[0], OpCode::OpReturn as u8);
    assert_eq!(chunk.code[1], OpCode::OpReturn as u8);
  }
}
