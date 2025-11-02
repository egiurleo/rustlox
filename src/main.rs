use crate::vm::{InterpretResult, VM};
use std::io::Write;
use std::{env, fs, io, process::exit};

mod chunk;
mod compiler;
mod debug;
mod helpers;
mod scanner;
mod value;
mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut vm = VM::new();

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&args[1], &mut vm);
    } else {
        eprintln!("Usage:...");
        exit(64);
    }
}

fn repl(vm: &mut VM) {
    loop {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        vm.interpret(input, &mut std::io::stdout());
    }
}

fn run_file(path: &String, vm: &mut VM) {
    let source = read_file(path);
    let result = vm.interpret(source, &mut std::io::stdout());

    if result == InterpretResult::CompileError {
        exit(65);
    }

    if result == InterpretResult::RuntimeError {
        exit(70);
    }
}

fn read_file(path: &String) -> String {
    match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            exit(74);
        }
    }
}
