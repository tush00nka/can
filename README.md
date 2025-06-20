# About
`can` is a stack-based programming language written in rust.
It is currently on the earliest stages of its development and basically copies the syntax of Tsoding's Porth. 

# Features
- addition
- substraction
- equality check
- dump (print number from the top of the stack)
- nestable if-else statements 
- while loops

# Basic program
While loop that prints numbers 10 to 1
```
10
dup 0 > 
while 
    dup .
    1 -
    dup 0 >
end
```

# Fibonacci sequence
```
mem 0 + 10 . // counter
mem 1 + 0 . // first
mem 2 + 1 . // second
mem 3 + 0 . // temp

// breaks after number 255 because we're using bytes :)
while mem dup , dup 0 > do
    1 - .
    mem 3 + mem 1 + , mem 2 + , + . // sum two numbers and put result in temp value
    mem 1 + mem 2 + , . // put second number in place of first
    mem 2 + mem 3 + , . // put temp value in place of second
    mem 3 + , dump // print
end
```

# Quick start
```
cargo run build examples/test.can && ./out
cargo run emulate examples/test.can
```