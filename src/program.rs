use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    process::exit,
};

use crate::operation::Operation;

pub struct Program {
    operations: Vec<Operation>,
    emulation_stack: Vec<u64>,
    emulation_mem: [u8; 640_000],
    opt_level: usize,
}

impl Program {
    pub fn from_file(file_path: &str, opt_level: usize) -> Self {
        let mut operations = vec![];

        let mut file: File = std::fs::File::open(file_path).expect("no such file");

        let mut code = String::new();
        let _ = file.read_to_string(&mut code);

        let mut includes = vec![];

        for line in code.lines() {
            if line.starts_with("include") {
                let mut parts = line.split_whitespace();
                parts.next();
                let include_path = parts
                    .next()
                    .expect("no include path provided")
                    .trim_matches('\"');
                let mut file = std::fs::File::open(include_path).expect("invalid include path");
                let mut code = String::new();
                let _ = file.read_to_string(&mut code);
                includes.push(code);
            }
        }

        let binding = code.clone();
        let lines: Vec<&str> = binding
            .lines()
            .into_iter()
            .filter(|&line| !line.starts_with("include"))
            .collect();

        code = "".to_owned();
        for include in includes {
            code += &include;
        }
        for line in lines {
            code += &(line.to_owned() + "\n");
        }

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
                    }
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
                    "+" => Operation::Plus { depth: 1 },
                    "-" => Operation::Minus,
                    "/" => Operation::Div,
                    "%" => Operation::Mod,
                    "=" => Operation::Equals,
                    "not" => Operation::Not,
                    "!=" => Operation::NotEqual,
                    ">" => Operation::Greater,
                    "<" => Operation::Lower,
                    "&" => Operation::BitAnd,
                    "|" => Operation::BitOr,
                    "<<" => Operation::BitShiftLeft,
                    ">>" => Operation::BitShiftRight,
                    "if" => Operation::If { address: 0 },
                    "else" => Operation::Else { address: 0 },
                    "while" => Operation::While,
                    "do" => Operation::Do { address: 0 },
                    "end" => Operation::End { address: 0 },
                    "dup" => Operation::Dup { depth: 1 },
                    "2dup" => Operation::Dup { depth: 2 },
                    "swap" => Operation::Swap,
                    "over" => Operation::Over,
                    "rot" => Operation::Rot,
                    "mem" => Operation::Mem,
                    "!" => Operation::Store,
                    "@" => Operation::Load,
                    "!32" => Operation::Store32,
                    "@32" => Operation::Load32,
                    "!64" => Operation::Store64,
                    "@64" => Operation::Load64,
                    "drop" => Operation::Drop,
                    "syscall1" => Operation::Syscall { arg_count: 1 },
                    "syscall2" => Operation::Syscall { arg_count: 2 },
                    "syscall3" => Operation::Syscall { arg_count: 3 },
                    "macro" => Operation::DefineMacro {
                        name: tokens
                            .next()
                            .expect("Macro isn't followed by a name!")
                            .1
                            .to_owned(),
                    },
                    ";" => Operation::EndMacro,
                    _ => {
                        if token.starts_with(|c: char| c.is_digit(10)) {
                            Operation::Push(
                                token
                                    .parse::<u64>()
                                    .expect(&format!("Unexpected operand '{token}'")),
                            )
                        } else {
                            Operation::CallMacro {
                                name: token.to_owned(),
                            }
                        }
                    }
                };

                operations.push(op);
            }
        }

        Self {
            operations,
            emulation_stack: vec![],
            emulation_mem: [0; 640_000],
            opt_level,
        }
    }

    pub fn cross_reference(&mut self) {
        let mut stack = vec![];

        let mut macros: HashMap<String, Vec<Operation>> = HashMap::new();
        let mut accumulating_macro = false;
        let mut macro_pool = vec![];
        let mut name_buf = "".to_string();

        let mut opsi = self.operations.clone().into_iter();

        let mut new_ops = vec![];

        while let Some(op) = opsi.next() {
            match op {
                Operation::DefineMacro { name, .. } => {
                    accumulating_macro = true;
                    name_buf = name;
                    macro_pool.clear();
                }
                Operation::EndMacro => {
                    if macros.contains_key(&name_buf) {
                        // TODO: better error message with line and col
                        println!("ERROR: Macro redefinition: `{}`", name_buf);
                        exit(1);
                    }
                    accumulating_macro = false;
                    macros.insert(name_buf.to_string(), macro_pool.clone());
                }
                _ => {
                    if accumulating_macro {
                        macro_pool.push(op.clone());
                    } else {
                        new_ops.push(op.clone());
                    }
                }
            }
        }

        self.operations = new_ops;

        fn unwrap_macro(
            name: &str,
            macros: &mut HashMap<String, Vec<Operation>>,
        ) -> Vec<Operation> {
            let ops = macros.get(name).unwrap().clone();
            ops.iter()
                .map(|op: &Operation| match op {
                    Operation::CallMacro { name } => {
                        return unwrap_macro(name, macros);
                    }
                    _ => return vec![op.clone()],
                })
                .flatten()
                .collect()
        }

        // unwrap all macros recursively
        self.operations = self
            .operations
            .iter()
            .map(|op| match op {
                Operation::CallMacro { name } => unwrap_macro(name, &mut macros),
                _ => {
                    vec![op.clone()]
                }
            })
            .flatten()
            .collect::<Vec<Operation>>();

        // fold all arithmetics for basic optimizations
        if self.opt_level > 0 {
            self.operations = crate::utils::fold_optimizations(self.operations.clone());
        }

        // actual cross-referencing
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

    pub fn emulate(&mut self) {
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
                Operation::Plus { depth } => {
                    let a = stack.pop().unwrap();
                    // let b = stack.pop().unwrap();
                    let mut acc = a;
                    for _ in 0..*depth {
                        acc += stack.pop().unwrap();
                    }
                    stack.push(acc);
                }
                Operation::Inc => {
                    let a = stack.pop().unwrap() + 1;
                    stack.push(a);
                }
                Operation::Minus => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
                Operation::Dec => {
                    let a = stack.pop().unwrap() - 1;
                    stack.push(a);
                }
                Operation::Div => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a / b);
                }
                Operation::Mod => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a % b);
                }
                Operation::Equals => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push((a == b) as u64);
                }
                Operation::Not => {
                    let a = stack.pop().unwrap();
                    stack.push(!a);
                }
                Operation::NotEqual => {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push((a != b) as u64);
                }
                Operation::Greater => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push((a > b) as u64);
                }
                Operation::Lower => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push((a < b) as u64);
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
                    let result = if b >= 64 { 0 } else { a << b };
                    stack.push(result);
                }
                Operation::BitShiftRight => {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = if b >= 64 { 0 } else { a >> b };
                    stack.push(result);
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
                Operation::Rot => {
                    if stack.len() > 2 {
                        let top = stack.pop().unwrap();
                        let middle = stack.pop().unwrap();
                        let bottom = stack.pop().unwrap();
                        stack.push(middle);
                        stack.push(top);
                        stack.push(bottom);
                    }
                }
                Operation::Drop => {
                    stack.pop();
                }
                Operation::Mem => {
                    stack.push(self.emulation_mem.as_ptr() as u64);
                }
                Operation::Store => {
                    // eprintln!("Store: stack before pop = {:?}", stack);
                    if stack.len() < 2 {
                        eprintln!("Store: stack underflow, stack={:?}", stack);
                        panic!("stack underflow");
                    }

                    let val = stack.pop().unwrap();
                    let addr = stack.pop().unwrap() as *mut u8;
                    if addr as u8 == 0 {
                        eprintln!("Store with address 0, val={}, stack={:?}", val, stack);
                        panic!("Null pointer store");
                    }
                    let data = unsafe { addr.as_mut().unwrap() };
                    *data = val as u8;
                }
                Operation::Load => {
                    if stack.len() < 1 {
                        eprintln!("Store: stack underflow, stack={:?}", stack);
                        panic!("stack underflow");
                    }
                    let mem_addr = stack.pop().unwrap() as *const u8;
                    stack.push(*unsafe { mem_addr.as_ref().unwrap() } as u64);
                }
                Operation::Store32 => unimplemented!(),
                Operation::Load32 => unimplemented!(),
                Operation::Store64 => unimplemented!(),
                Operation::Load64 => unimplemented!(),
                Operation::Syscall { arg_count } => {
                    let syscall_code = stack.pop().unwrap() as u64;
                    let mut args = vec![];

                    for _ in 0..*arg_count {
                        let val = stack.pop().unwrap();
                        args.push(val as u64);
                    }

                    let (a1, a2, a3) = match args.as_slice() {
                        &[a1, a2, a3] => (a1, a2, a3),
                        _ => {
                            panic!("this should never happen")
                        }
                    };
                    unsafe {
                        crate::utils::syscall_3(syscall_code, a1, a2, a3);
                    }
                }
                Operation::String(_) => unimplemented!(),
                Operation::CallMacro { .. } => {}
                Operation::DefineMacro { .. } => {}
                Operation::EndMacro => {}
            }
        }
    }

    pub fn compile(&mut self) {
        self.cross_reference();

        let mut code = "global _start\nsection .text\n".to_owned();

        code.push_str("_start:\n");

        for i in 0..self.operations.len() {
            let op = self.operations.get(i).unwrap();
            let operation = op.to_x86_64(i);
            code.push_str(&operation);
        }

        code.push_str("\tmov rax, 60\n\tmov rdi, 0\n\tsyscall\n\n");
        // TODO: remove hardcoded buffer size
        code.push_str(&format!(
            "section .bss\nmem resb {}\n",
            crate::MEM_BUFFER_SIZE
        ));

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
