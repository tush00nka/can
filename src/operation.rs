#[derive(PartialEq, Clone, Debug)]
pub enum Operation {
    Push(u64),
    String(String),
    Plus { depth: usize },
    Inc,
    Minus,
    Dec,
    NotEqual,
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
    Rot,
    Mem,
    Store,
    Load,
    Store32,
    Load32,
    Store64,
    Load64,
    Drop,
    Syscall { arg_count: usize },
    DefineMacro { name: String },
    EndMacro,
    CallMacro { name: String },
}

impl Operation {
    pub fn to_x86_64(&self, index: usize) -> String {
        match self {
            Operation::Push(number) => format!(";; -- {} --\n\tpush {}\n", number, number),
            Operation::Plus { depth } => {
                let mut op = ";; -- + --\n".to_owned() + "\tpop rax\n";
                for _ in 0..*depth {
                    op += "\tpop rbx\n\tadd rax, rbx\n"
                }
                op += "\tpush rax\n";

                op
            }
            Operation::Inc => {
                "".to_owned() + ";; -- INC --\n" + "\tpop rax\n" + "\tinc rax\n" + "\tpush rax\n"
            }
            Operation::Minus => {
                "".to_owned()
                    + ";; -- - --\n"
                    + "\tpop rbx\n"
                    + "\tpop rax\n"
                    + "\tsub rax, rbx\n"
                    + "\tpush rax\n"
            }
            Operation::Dec => {
                "".to_owned() + ";; -- DEC --\n" + "\tpop rax\n" + "\tdec rax\n" + "\tpush rax\n"
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
            Operation::NotEqual => {
                "".to_owned()
                    + ";; -- = --\n"
                    + "\tpop rax\n"
                    + "\tpop rbx\n"
                    + "\tcmp rax, rbx\n"
                    + "\tsetne al\n"
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
                    + ";; -- & --\n"
                    + "\tpop rbx\n"
                    + "\tpop rax\n"
                    + "\nand rax, rbx\n"
                    + "\tpush rax\n"
            }
            Operation::BitOr => {
                "".to_owned()
                    + ";; -- | --\n"
                    + "\tpop rbx\n"
                    + "\tpop rax\n"
                    + "\nor rax, rbx\n"
                    + "\tpush rax\n"
            }
            Operation::BitShiftLeft => {
                "".to_owned()
                    + ";; -- << --\n"
                    + "\tpop rcx\n"
                    + "\tpop rax\n"
                    + "\nshl rax, cl\n"
                    + "\tpush rax\n"
            }
            Operation::BitShiftRight => {
                "".to_owned()
                    + ";; -- >> --\n"
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
                    + &format!("label_{index}:\n")
            }
            Operation::While => {
                format!(";; -- WHILE -- \nlabel_{index}:\n")
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
                    format!(";; -- END WHILE -- \n\tjmp label_{address}\nlabel_{index}:\n")
                } else {
                    format!(";; -- END IF -- \n\tlabel_{index}:\n")
                }
            }
            Operation::EndMacro => "".to_owned(),
            Operation::Dup { depth } => {
                if *depth == 1 {
                    "".to_owned()
                        + ";; -- DUP --\n"
                        + "\tpop rax\n"
                        + "\tpush rax\n"
                        + "\tpush rax\n"
                } else if *depth == 2 {
                    "".to_owned()
                        + ";; -- DUP 2 --\n"
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
                ";; -- SWAP --\n".to_owned()
                    + "\tpop rax\n"
                    + "\tpop rbx\n"
                    + "\tpush rax\n"
                    + "\tpush rbx\n"
            }
            Operation::Over => {
                ";; -- OVER --\n".to_owned()
                    + "\tpop rax\n"
                    + "\tpop rbx\n"
                    + "\tpush rbx\n"
                    + "\tpush rax\n"
                    + "\tpush rbx\n"
            }
            Operation::Rot => {
                ";; -- ROT --\n".to_owned()
                    + "\tpop rax\n"
                    + "\tpop rbx\n"
                    + "\tpop rdi\n"
                    + "\tpush rbx\n"
                    + "\tpush rax\n"
                    + "\tpush rdi\n"
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
            Operation::Store32 => {
                "".to_owned()
                    + ";; -- STORE_32 --\n"
                    + "\tpop rbx\n"
                    + "\tpop rax\n"
                    + "\tmov [rax], ebx\n"
            }
            Operation::Load32 => {
                "".to_owned()
                    + ";; -- LOAD_32 --\n"
                    + "\tpop rax\n"
                    + "\txor rbx, rbx\n"
                    + "\tmov ebx, [rax]\n"
                    + "\tpush rbx\n"
            }
            Operation::Store64 => {
                "".to_owned()
                    + ";; -- STORE_64 --\n"
                    + "\tpop rbx\n"
                    + "\tpop rax\n"
                    + "\tmov [rax], rbx\n"
            }
            Operation::Load64 => {
                "".to_owned()
                    + ";; -- LOAD_54 --\n"
                    + "\tpop rax\n"
                    + "\txor rbx, rbx\n"
                    + "\tmov rbx, [rax]\n"
                    + "\tpush rbx\n"
            }
            Operation::Syscall { arg_count } => {
                let mut syscall_code =
                    "".to_owned() + &format!(";; -- SYSCALL {arg_count} --\n") + "\tpop rax\n"; // syscall number

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
            Operation::DefineMacro { .. } => {
                "".to_owned()
                // format!(";; -- A FUNCTION '{name}' HAS BEEN DEFINED HERE --\n")
            }
            Operation::CallMacro { .. } => "".to_owned(),
        }
    }
}