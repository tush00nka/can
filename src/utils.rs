use std::arch::asm;

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