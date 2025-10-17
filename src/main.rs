use crate::chunk::{init_chunk, OpCode};
use crate::debug::disassemble_chunk;

mod debug;

mod chunk;
mod value;

fn main() {
    let mut chunk = init_chunk();
    chunk.write(OpCode::OpReturn as u8);

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8);
    chunk.write(constant as u8);

    disassemble_chunk(chunk, "test chunk", &mut std::io::stdout());
}
