use core::iter::Iterator;

use crate::program::Program;

mod operation;
mod program;
mod utils;

const MEM_BUFFER_SIZE: usize = 640_000;

fn usage() {
    println!(
        "USAGE: can <COMMAND> <FILE>\nCommands:\n\tbuild\t\tbuild a binary\n\temulate\t\temulate the program execution"
    );
}

fn main() {
    let mut args = std::env::args().enumerate();
    let (_, _program_name) = args.next().unwrap();
    let Some((_, mode)) = args.next() else {
        usage();
        println!("ERROR: No command provided");
        return;
    };
    let Some((_, file_path)) = args.next() else {
        usage();
        println!("ERROR: No source file provided");
        return;
    };

    // TODO: make proper cli args parsing and optimization level flag (-O)
    let mut program = Program::from_file(&file_path, 1); // hard-code the optimization level for now

    match mode.as_str() {
        "build" => program.compile(),
        "emulate" => program.emulate(),
        _ => {
            usage();
            println!("ERROR: Unknown command");
        }
    };
}
