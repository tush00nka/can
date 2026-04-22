use std::arch::asm;

use crate::Operation;

// taken from: https://phip1611.de/blog/direct-systemcalls-to-linux-from-rust-code-x86_64/
pub unsafe fn syscall_3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 { unsafe {
    let res;
    asm!(
        // there is no need to write "mov"-instructions, see below
        "syscall",
        // from 'in("rax")' the compiler will
        // generate corresponding 'mov'-instructions
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") res,
    );
    res
}}

pub fn fold_optimizations(ops: Vec<Operation>) -> Vec<Operation> {
    ops.into_iter().fold(Vec::new(), |mut acc, op| {
        match (acc.last(), &op) {
            // Push(1) followed by Plus { depth: 1 } -> Inc
            (Some(Operation::Push(1)), Operation::Plus { depth: 1 }) => {
                acc.pop();  // remove the Push(1)
                acc.push(Operation::Inc);
            }
            // Same for decrement
            (Some(Operation::Push(1)), Operation::Minus) => {
                acc.pop();  // remove the Push(1)
                acc.push(Operation::Dec);
            }
            // Consecutive Plus variants
            (Some(Operation::Plus { depth: last_depth }), Operation::Plus { depth }) => {
                *acc.last_mut().unwrap() = Operation::Plus { depth: last_depth + depth };
            }
            // Everything else just push
            _ => acc.push(op),
        }
        acc
    })
}