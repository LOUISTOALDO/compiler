//! C Compiler CLI - Compiles C source to x86-64 assembly.

use anyhow::Result;
use c_compiler::{lex, CodeGen, Parser};
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.c> [--tokens]", args[0]);
        eprintln!("  Compiles C source to x86-64 assembly (output: input.s)");
        std::process::exit(1);
    }

    let debug_tokens = args.contains(&"--tokens".to_string());
    let input_path = args.iter().skip(1).find(|a| !a.starts_with('-')).unwrap_or_else(|| &args[1]);
    let source = fs::read_to_string(input_path)?;

    // Lex
    let tokens = lex(&source).map_err(|e| anyhow::anyhow!("Lex error: {}", e))?;

    if debug_tokens {
        for (i, t) in tokens.iter().enumerate() {
            println!("{}: {:?}", i, t);
        }
        return Ok(());
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Codegen
    let mut codegen = CodeGen::new();
    let asm = codegen.compile(&program);

    // Output: input.c -> input.s
    let output_path = Path::new(input_path).with_extension("s");
    fs::write(&output_path, asm)?;

    println!("Compiled {} -> {}", input_path, output_path.display());
    println!("Assemble: gcc {} -o program && ./program", output_path.display());

    Ok(())
}
