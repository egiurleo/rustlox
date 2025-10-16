use crate::chunk::{init_chunk, OpCode};
use crate::debug::disassemble_chunk;

mod chunk;
mod debug;

fn main() {
    let mut chunk = init_chunk();
    chunk.write(OpCode::OpReturn as u8);

    disassemble_chunk(chunk, "test chunk", &mut std::io::stdout());
}
