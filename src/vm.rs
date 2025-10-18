use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use std::io::Write;

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

static mut VM_INSTANCE: Option<VM> = None;

pub fn init_vm() {
  unsafe {
    VM_INSTANCE = Some(Default::default());
  }
}

pub fn interpret<W: Write>(chunk: Chunk, writer: &mut W) -> InterpretResult {
  unsafe {
    if let Some(vm) = VM_INSTANCE.as_mut() {
      vm.chunk = chunk;
      vm.ip = 0;
    }
  }

  run(writer)
}

fn run<W: Write>(writer: &mut W) -> InterpretResult {
  let mut instruction: u8;

  loop {
    instruction = read_byte();
    if instruction == OpCode::OpConstant as u8 {
      let constant = read_constant();
      writeln!(writer, "{}", constant).unwrap();
    } else if instruction == OpCode::OpReturn as u8 {
      return InterpretResult::InterpretOk;
    }
  }
}

#[inline]
fn read_byte() -> u8 {
  unsafe {
    if let Some(vm) = VM_INSTANCE.as_mut() {
      let byte = vm.chunk.code[vm.ip as usize];
      vm.ip = vm.ip + 1;
      return byte;
    } else {
      panic!("VM_INSTANCE not initialized");
    }
  }
}

#[inline]
fn read_constant() -> Value {
  unsafe {
    if let Some(vm) = VM_INSTANCE.as_mut() {
      vm.chunk.constants.at(read_byte() as usize)
    } else {
      panic!("VM_INSTANCE not initialized");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_vm_test() {
    unsafe {
      assert!(VM_INSTANCE.is_none());
      init_vm();
      assert!(VM_INSTANCE.is_some());
    }
  }

  #[test]
  fn interpret_test() {
    init_vm();

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant as u8, 123);
    chunk.write(OpCode::OpReturn as u8, 123);

    let mut output = Vec::new();
    let result = interpret(chunk, &mut output);

    let output_str = String::from_utf8(output).unwrap();

    match result {
      InterpretResult::InterpretOk => assert!(true),
    }

    assert_eq!(output_str, "1.2\n");
  }
}
