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
```
34 35 + 69 = if
    400 20 + 1000 = if 
        420 .
    else 
        0 .
    end
else
    69 .
end
```

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

# Quick start
```
cargo run build examples/test.can && ./out
cargo run emulate examples/test.can
```