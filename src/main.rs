use core::{iter::Iterator, unimplemented};
use std::{
    fs::File,
    io::{Read, Write},
};

#[derive(PartialEq, Clone)]
enum Operation {
    Push(i32),
    String(String),
    Plus,
    Minus,
    Equals,
    Greater,
    Lower,
    BitOr,
    BitAnd,
    BitShiftLeft,
    BitShiftRight,
    Dump,
    If { address: usize },
    Else { address: usize },
    While,
    Do { address: usize },
    End { address: usize },
    Dup { depth: usize },
    Swap,
    Over,
    Mem,
    Store,
    Load,
    Drop,
    Syscall { arg_count: usize },
}

const MEM_BUFFER_SIZE: usize = 640_000;

struct Program {
    operations: Vec<Operation>,
    emulation_stack: Vec<i32>,
}

impl Program {
    fn from_file(file_path: &str) -> Self {
        let mut operations = vec![];

        let mut file: File = std::fs::File::open(file_path).expect("no such file");

        let mut code = String::new();
        let _ = file.read_to_string(&mut code);

        for line in code.lines() {
            let mut line = line.trim().to_string();

            let mut reading_string = false;
            let mut escape_next = false;
            let mut strings: Vec<String> = vec![];
            let mut accum_string = "".to_owned();
            for ch in line.chars() {
                match ch {
                    '\"' => {
                        if reading_string {
                            strings.push(accum_string);
                            accum_string = "".to_owned();
                        } 
                        reading_string = !reading_string;
                        continue;
                    },
                    '\\' => {
                        escape_next = true;
                        continue;
                    }
                    'n' => {
                        if escape_next {
                            accum_string.push('\n');
                            escape_next = false;
                            continue;
                        }
                    }
                    _ => {}
                }

                if reading_string {
                    accum_string.push(ch);
                }
            }

            for string in strings.iter() {
                line = line.replacen(&format!("\\n"), "\n", 2);
                line = line.replace(&format!("\"{string}\""), "string");
            }

            let no_comments = line.split("//").next().unwrap();
            let mut tokens = no_comments.split_whitespace().enumerate();
            while let Some((_, token)) = tokens.next() {
                let op = match token {
                    "string" => Operation::String(strings.pop().unwrap()),
                    "+" => Operation::Plus,
                    "-" => Operation::Minus,
                    "=" => Operation::Equals,
                    ">" => Operation::Greater,
                    "<" => Operation::Lower,
                    "&" => Operation::BitAnd,
                    "|" => Operation::BitOr,
                    "<<" => Operation::BitShiftLeft,
                    ">>" => Operation::BitShiftRight,
                    "dump" => Operation::Dump,
                    "if" => Operation::If { address: 0 },
                    "else" => Operation::Else { address: 0 },
                    "while" => Operation::While,
                    "do" => Operation::Do { address: 0 },
                    "end" => Operation::End { address: 0 },
                    "dup" => Operation::Dup { depth: 1 },
                    "2dup" => Operation::Dup { depth: 2 },
                    "swap" => Operation::Swap,
                    "over" => Operation::Over,
                    "mem" => Operation::Mem,
                    "." => Operation::Store,
                    "," => Operation::Load,
                    "drop" => Operation::Drop,
                    "syscall1" => Operation::Syscall { arg_count: 1 },
                    "syscall3" => Operation::Syscall { arg_count: 3 },
                    _ => Operation::Push(
                        token
                            .parse::<i32>()
                            .expect(&format!("Unexpected operand '{token}'")),
                    ),
                };

                operations.push(op);
            }
        }

        Self {
            operations,
            emulation_stack: vec![],
        }
    }

    fn cross_reference(&mut self) {
        let mut stack = vec![];
        let test_ops = self.operations.clone();
        for i in 0..self.operations.len() {
            let op = self.operations.get_mut(i).unwrap();
            match op {
                Operation::If { .. } => {
                    stack.push(i);
                }
                Operation::Else { .. } => {
                    // TODO: check for end of stack and throw error if extra `else`
                    let op_id = stack.pop().unwrap();
                    match self.operations.get_mut(op_id).unwrap() {
                        Operation::If { address } => {
                            *address = i;
                        }
                        _ => {}
                    }
                    stack.push(i);
                }
                Operation::While => {
                    stack.push(i);
                }
                Operation::Do { .. } => {
                    stack.push(i);
                }
                Operation::End { address } => {
                    // TODO: check for end of stack and throw error if extra `end`
                    let op_id = stack.pop().unwrap();
                    let cross_op = test_ops[op_id].clone();
                    if cross_op == (Operation::Do { address: 0 }) {
                        *address = stack.pop().unwrap();
                    }
                    match self.operations.get_mut(op_id).unwrap() {
                        Operation::If { address } => {
                            *address = i;
                        }
                        Operation::Else { address } => {
                            *address = i;
                        }
                        Operation::Do { address } => {
                            *address = i;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn emulate(&mut self) {
        self.cross_reference();

        let stack = &mut self.emulation_stack;
        let mut pointer = 0;
        while pointer < self.operations.len() {
            let op = self.operations.get(pointer).unwrap();
            pointer += 1;
            match op {
                Operation::Push(number) => {
                    stack.push(*number);
                }
                Operation::Plus => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(a + b);
                }
                Operation::Minus => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
                Operation::Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push((a == b) as i32);
                }
                Operation::Greater => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push((a > b) as i32);
                }
                Operation::Lower => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push((a < b) as i32);
                }
                Operation::BitAnd => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a & b);
                }
                Operation::BitOr => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a | b);
                }
                Operation::BitShiftLeft => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a << b);
                }
                Operation::BitShiftRight => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a >> b);
                }
                Operation::Dump => {
                    println!("{}", stack.pop().unwrap());
                }
                Operation::If { address } => {
                    // true if not 0,
                    // so 1, 2, 69, -420 etc all count as 1
                    if stack.pop().unwrap() == 0 {
                        // TODO: check for address bounds
                        pointer = *address + 1;
                    }
                }
                Operation::Else { address } => {
                    pointer = *address + 1;
                }
                Operation::While => {}
                Operation::Do { address } => {
                    if stack.pop().unwrap() == 0 {
                        // TODO: check for address bounds
                        pointer = *address + 1;
                    }
                }
                Operation::End { address } => {
                    if *address > 0 {
                        pointer = *address;
                    }
                }
                Operation::Dup { depth } => {
                    // if stack has something ontop, push it's copy onto the stack
                    if stack.len() >= *depth {
                        if *depth == 1 {
                            stack.push(stack[stack.len() - 1]);
                        } else if *depth == 2 {
                            let val1 = stack.pop().unwrap();
                            let val2 = stack.pop().unwrap();
                            stack.push(val2);
                            stack.push(val1);
                            stack.push(val2);
                            stack.push(val1);
                        }
                    }
                }
                Operation::Swap => {
                    if stack.len() > 1 {
                        let last = stack.pop().unwrap();
                        let prev = stack.pop().unwrap();
                        stack.push(prev);
                        stack.push(last);
                    }
                }
                Operation::Over => {
                    if stack.len() > 1 {
                        let top = stack.pop().unwrap();
                        let over = stack.pop().unwrap();
                        stack.push(over);
                        stack.push(top);
                        stack.push(over);
                    }
                }
                Operation::Drop => {
                    stack.pop();
                }
                Operation::Mem => {
                    unimplemented!()
                }
                Operation::Store => {
                    unimplemented!()
                }
                Operation::Load => {
                    unimplemented!()
                }
                Operation::Syscall { .. } => {
                    unimplemented!()
                }
                Operation::String(_) => unimplemented!(),
            }
        }
    }

    fn compile(&mut self) {
        self.cross_reference();

        let mut code = "global _start\nsection .text\n".to_owned();

        let dump = [
            "dump:\n",
            "    mov     r9, -3689348814741910323\n",
            "    sub     rsp, 40\n",
            "    mov     BYTE [rsp+31], 10\n",
            "    lea     rcx, [rsp+30]\n",
            ".L2:\n",
            "    mov     rax, rdi\n",
            "    lea     r8, [rsp+32]\n",
            "    mul     r9\n",
            "    mov     rax, rdi\n",
            "    sub     r8, rcx\n",
            "    shr     rdx, 3\n",
            "    lea     rsi, [rdx+rdx*4]\n",
            "    add     rsi, rsi\n",
            "    sub     rax, rsi\n",
            "    add     eax, 48\n",
            "    mov     BYTE [rcx], al\n",
            "    mov     rax, rdi\n",
            "    mov     rdi, rdx\n",
            "    mov     rdx, rcx\n",
            "    sub     rcx, 1\n",
            "    cmp     rax, 9\n",
            "    ja      .L2\n",
            "    lea     rax, [rsp+32]\n",
            "    mov     edi, 1\n",
            "    sub     rdx, rax\n",
            "    xor     eax, eax\n",
            "    lea     rsi, [rsp+32+rdx]\n",
            "    mov     rdx, r8\n",
            "    mov     rax, 1\n",
            "    syscall\n",
            "    add     rsp, 40\n",
            "    ret\n",
        ];

        for line in dump {
            code.push_str(line);
        }

        code.push_str("_start:\n");

        for i in 0..self.operations.len() {
            let op = self.operations.get(i).unwrap();
            let operation = match op {
                Operation::Push(number) => format!(";; -- {} --\n\tpush {}\n", number, number),
                Operation::Plus => {
                    "".to_owned()
                        + ";; -- + --\n"
                        + "\tpop rax\n"
                        + "\tpop rbx\n"
                        + "\tadd rax, rbx\n"
                        + "\tpush rax\n"
                }
                Operation::Minus => {
                    "".to_owned()
                        + ";; -- - --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\tsub rax, rbx\n"
                        + "\tpush rax\n"
                }
                Operation::Equals => {
                    "".to_owned()
                                    + ";; -- = --\n"
                                    + "\tpop rax\n"
                                    + "\tpop rbx\n"
                                    + "\tcmp rax, rbx\n"
                                    + "\tsete al\n"
                                    // + "\tmovzx rax, al\n"
                                    + "\tpush rax\n"
                }
                Operation::Greater => {
                    "".to_owned()
                                    + ";; -- > --\n"
                                    + "\tpop rbx\n"
                                    + "\tpop rax\n"
                                    + "\tcmp rax, rbx\n"
                                    + "\tsetg al\n"
                                    // + "\tmovzx rax, al\n"
                                    + "\tpush rax\n"
                }
                Operation::Lower => {
                    "".to_owned()
                                    + ";; -- < --\n"
                                    + "\tpop rbx\n"
                                    + "\tpop rax\n"
                                    + "\tcmp rax, rbx\n"
                                    + "\tsetl al\n"
                                    // + "\tmovzx rax, al\n"
                                    + "\tpush rax\n"
                }
                Operation::BitAnd => {
                    "".to_owned()
                        + ";; -- < --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\nand rax, rbx\n"
                        + "\tpush rax\n"
                }
                Operation::BitOr => {
                    "".to_owned()
                        + ";; -- < --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\nor rax, rbx\n"
                        + "\tpush rax\n"
                }
                Operation::BitShiftLeft => {
                    "".to_owned()
                        + ";; -- < --\n"
                        + "\tpop rcx\n"
                        + "\tpop rax\n"
                        + "\nshl rax, cl\n"
                        + "\tpush rax\n"
                }
                Operation::BitShiftRight => {
                    "".to_owned()
                        + ";; -- < --\n"
                        + "\tpop rcx\n"
                        + "\tpop rax\n"
                        + "\nshr rax, cl\n"
                        + "\tpush rax\n"
                }
                Operation::Dump => "".to_owned() + "\tpop rdi\n" + "\tcall dump\n",
                Operation::If { address } => {
                    "".to_owned()
                        + ";; -- IF -- \n"
                        + "\tpop rax\n"
                        + "\ttest rax, rax\n"
                        + &format!("\tjz label_{address}\n")
                }
                Operation::Else { address } => {
                    "".to_owned()
                        + ";; -- ELSE -- \n"
                        + &format!("\tjmp label_{address}\n")
                        + &format!("label_{i}:\n")
                }
                Operation::While => {
                    format!(";; -- WHILE -- \nlabel_{i}:\n")
                }
                Operation::Do { address } => {
                    "".to_owned()
                        + ";; -- DO -- \n"
                        + "\tpop rax\n"
                        + "\ttest rax, rax\n"
                        + &format!("\tjz label_{address}\n")
                }
                Operation::End { address } => {
                    if *address > 0 {
                        format!(";; -- END WHILE -- \n\tjmp label_{address}\nlabel_{i}:\n")
                    } else {
                        format!(";; -- END IF -- \n\tlabel_{i}:\n")
                    }
                }
                Operation::Dup { depth } => {
                    if *depth == 1 {
                        "".to_owned() + "\tpop rax\n" + "\tpush rax\n" + "\tpush rax\n"
                    } else if *depth == 2 {
                        "".to_owned()
                            + "\tpop rax\n"
                            + "\tpop rbx\n"
                            + "\tpush rbx\n"
                            + "\tpush rax\n"
                            + "\tpush rbx\n"
                            + "\tpush rax\n"
                    } else {
                        "".to_owned()
                    }
                }
                Operation::Swap => {
                    "".to_owned() + "\tpop rax\n" + "\tpop rbx\n" + "\tpush rax\n" + "\tpush rbx\n"
                }
                Operation::Over => {
                    "".to_owned()
                        + "\tpop rax\n"
                        + "\tpop rbx\n"
                        + "\tpush rbx\n"
                        + "\tpush rax\n"
                        + "\tpush rbx\n"
                }
                Operation::Drop => "".to_owned() + "\tpop rax\n" + "\txor rax, rax\n",
                Operation::Mem => "".to_owned() + ";; -- MEM --\n" + "\tpush mem\n",
                Operation::Store => {
                    "".to_owned()
                        + ";; -- STORE --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\tmov [rax], bl\n"
                }
                Operation::Load => {
                    "".to_owned()
                        + ";; -- LOAD --\n"
                        + "\tpop rax\n"
                        + "\txor rbx, rbx\n"
                        + "\tmov bl, [rax]\n"
                        + "\tpush rbx\n"
                }
                Operation::Syscall { arg_count } => {
                    let mut syscall_code = "".to_owned() + ";; -- SYSCALL 3 --\n" + "\tpop rax\n"; // syscall number

                    let arg_registers = ["rdi", "rsi", "rdx", "r10", "r8", "r9"];

                    for i in 0..*arg_count {
                        syscall_code += &format!("\tpop {}\n", arg_registers[i]);
                    }

                    syscall_code += "\tsyscall\n";

                    syscall_code
                }
                Operation::String(string) => {
                    let mut op = "".to_owned();

                    for (offset, b) in string.as_bytes().iter().enumerate() {
                        op += &format!("\tmov [mem+{offset}], BYTE {b}\n");
                    }

                    op += &format!("\tpush {}\n", string.len());
                    op += "\tpush mem\n";

                    op
                }
            };
            code.push_str(&operation);
        }

        code.push_str("\tmov rax, 60\n\tmov rdi, 0\n\tsyscall\n\n");
        // TODO: remove hardcoded buffer size
        code.push_str(&format!("section .bss\nmem resb {}\n", MEM_BUFFER_SIZE));

        let mut file = std::fs::File::create("./out.asm").expect("Failed to create .asm file");
        file.write_all(code.as_bytes())
            .expect("Failed to write to .asm file");
        let _ = std::process::Command::new("nasm")
            .args(["-felf64", "out.asm"])
            .output()
            .expect("failed to compile .asm code");
        let _ = std::process::Command::new("ld")
            .args(["-o", "out", "out.o"])
            .output()
            .expect("failed to link object file");
    }
}

fn usage() {
    println!(
        "USAGE: can <COMMAND> <FILE>\nCommands:\n\tbuild\t\tbuild a binary\n\temulate\t\temulate the program execution"
    );
}

fn main() {
    // let mut program = Program::new(vec![
    //     Operation::Push(34),
    //     Operation::Push(35),
    //     Operation::Plus,
    //     Operation::Dump,
    //     Operation::Push(500),
    //     Operation::Push(80),
    //     Operation::Minus,
    //     Operation::Dump,
    // ]);

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

    let mut program = Program::from_file(&file_path);

    match mode.as_str() {
        "build" => program.compile(),
        "emulate" => program.emulate(),
        _ => {
            usage();
            println!("ERROR: Unknown command");
        }
    };
}
