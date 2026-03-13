# C Compiler (Rust)

A minimal C compiler written in Rust. Compiles a subset of C to x86-64 assembly (Linux System V ABI).

## Supported C Subset

- **Types:** `int` only
- **Variables:** `int x;`, `int x = expr;`
- **Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!`
- **Control flow:** `if`/`else`, `while`, `for`
- **Functions:** definitions, calls, recursion
- **Comments:** `//` and `/* */`

## Build & Run

```bash
cargo build
./target/debug/c-compiler examples/simple.c
```

Output: `examples/simple.s` (assembly). Assemble with:

```bash
gcc examples/simple.s -o program && ./program
```

## Examples

| File | Description |
|------|-------------|
| `examples/simple.c` | Variables, arithmetic |
| `examples/fib.c` | Recursive Fibonacci |
| `examples/minimal.c` | Minimal `return 0` |

## Project Structure

```
src/
├── main.rs    # CLI
├── lib.rs     # Module exports
├── lexer.rs   # Tokenizer (logos)
├── parser.rs  # Recursive descent parser
├── ast.rs     # Abstract syntax tree
└── codegen.rs # x86-64 assembly emitter
```

## Dependencies

- `logos` - lexer
- `anyhow` / `thiserror` - error handling
