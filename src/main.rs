use std::io::{Read, Write};

#[derive(PartialEq, Clone, Copy)]
enum Operation {
    Push(i32),
    Plus,
    Minus,
    Equals,
    Greater,
    Lower,
    Dump,
    If { address: usize },
    Else { address: usize },
    While,
    Do { address: usize },
    End { address: usize },
    Dup,
    Mem,
    Store,
    Load,
}

const MEM_BUFFER_SIZE: usize = 640_000;

struct Program {
    operations: Vec<Operation>,
    emulation_stack: Vec<i32>,
}

impl Program {
    #[allow(unused)]
    fn new(operations: Vec<Operation>) -> Self {
        Self {
            operations,
            emulation_stack: vec![],
        }
    }

    fn from_file(file_path: &str) -> Self {
        let mut operations = vec![];

        let mut file = std::fs::File::open(file_path).expect("no such file");

        let code = &mut String::new();
        let _ = file.read_to_string(code);

        for line in code.lines() {
            let line = line.trim();
            let no_comments = line.split("//").next().unwrap();
            let mut tokens = no_comments.split_whitespace().enumerate();
            while let Some((_, token)) = tokens.next() {
                operations.push(match token {
                    "+" => Operation::Plus,
                    "-" => Operation::Minus,
                    "=" => Operation::Equals,
                    ">" => Operation::Greater,
                    "<" => Operation::Lower,
                    "dump" => Operation::Dump,
                    "if" => Operation::If { address: 0 },
                    "else" => Operation::Else { address: 0 },
                    "while" => Operation::While,
                    "do" => Operation::Do { address: 0 },
                    "end" => Operation::End { address: 0 },
                    "dup" => Operation::Dup,
                    "mem" => Operation::Mem,
                    "." => Operation::Store,
                    "," => Operation::Load,
                    _ => Operation::Push(
                        token
                            .parse::<i32>()
                            .expect(&format!("Unexpected operand '{token}'")),
                    ),
                });
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
                    let cross_op = test_ops[op_id];
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
                Operation::Dup => {
                    // if stack has something ontop, push it's copy onto the stack
                    if stack.len() > 0 {
                        stack.push(stack[stack.len() - 1]);
                    }
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
                        + "\tmovzx rax, al\n"
                        + "\tpush rax\n"
                }
                Operation::Greater => {
                    "".to_owned()
                        + ";; -- > --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\tcmp rax, rbx\n"
                        + "\tsetg al\n"
                        + "\tmovzx rax, al\n"
                        + "\tpush rax\n"
                }
                Operation::Lower => {
                    "".to_owned()
                        + ";; -- < --\n"
                        + "\tpop rbx\n"
                        + "\tpop rax\n"
                        + "\tcmp rax, rbx\n"
                        + "\tsetl al\n"
                        + "\tmovzx rax, al\n"
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
                Operation::Dup => "".to_owned() + "\tpop rax\n" + "\tpush rax\n" + "\tpush rax\n",
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
