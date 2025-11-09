# LilCC Compiler

**LilCC** (Little C Compiler) is a simple C compiler that currently supports a minimal subset of the C language.  
It is designed for learning, experimentation, and understanding how high level code maps to assembly.

---

## Features

### ✅ Supported
- Tpyes supported are `long` and `int`
- Function definitions and calls
- Control flow:
  - `if` / `else`
  - `while`
  - `do-while`
  - `for`
  - `return`
- Expressions:
  - Arithmetic operations (`+`, `-`, `*`, `/`, `%`)
  - comparison operations (`<`, `>`, `==`, `!=`, etc.)
  - logical operations(`!`, `&&`, `||`)
  - Unary operations (`-`, `!`)
  - Conditional expressions (`?:`)
  - Variable assignment
- Local variables inside functions
- Compilation to assembly and object files
- Global and static variables declarations and definitions
- Error reporting with source code spans

---

## Command Line Usage

```bash
lilcc <FILE> [OPTIONS]
```

**<FILE>**  
The source file to compile (C code).

**[OPTIONS]** can be:

- `--asm` : Output the assembly file instead of linking.
- `-c`    : Output the object file instead of linking.
