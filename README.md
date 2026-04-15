# About
`can` is a stack-based programming language written in Rust.
It is currently at the earliest stages of its development and basically copies the syntax of Tsoding's Porth. 

# Features
- Arithmetic operations
- Bitwise logic operations
- Dump (print number from the top of the stack)
- if-else statements and while-loops
- Memory addressing
- Syscalls
- Macros
- C-style comments (//)
- [Turing complete](./examples/rule110.can)

# Basic program
While loop that prints numbers 1 to 10
```
10 0 while 2dup > do 
    1 +
    dup dump
end
```

# Fibonacci sequence
```
// prints first 10 numbers of Fibonacci sequence

macro BYTE 8 ;

macro first mem ;
macro second mem BYTE + ;
macro temp mem BYTE BYTE + + ;

first  0 !64 // first
second 1 !64 // second
temp   0 !64 // temp

20 0 while 2dup > do
    1 +
    temp first @64 second @64 + !64 // sum two numbers and put result in temp value
    first second @64 !64            // put second number in place of first
    second temp @64 !64             // put temp value in place of second
    temp @64 dump                   // print
end
```

# Quick start
```
cargo run -- build examples/test.can && ./out
cargo run -- emulate examples/test.can
```
