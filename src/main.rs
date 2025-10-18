use crate::chunk::{OpCode, Chunk};
use crate::debug::disassemble_chunk;

mod debug;

mod chunk;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    disassemble_chunk(chunk, "test chunk", &mut std::io::stdout());
}
