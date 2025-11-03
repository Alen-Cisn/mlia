mod codegen;
mod parser;
mod tokenizer;

use codegen::CodeGen;
use inkwell::context::Context;
use parser::parse_program;
use std::env::args;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        return Err("Please provide an input file as a command line argument.".into());
    }

    let input_file = &args[1];
    // By default we will compile to an executable whose name is the input file's
    // basename (without extension). The user can override this with --output/-o.
    let input_path = std::path::Path::new(input_file);
    let default_out = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("a.out")
        .to_string();
    let mut output_file: Option<String> = Some(default_out);

    // Parse command line arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a filename".into());
                }
            }
            "--jit" => {
                output_file = None; // Disable output file, use JIT execution
                i += 1;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]).into());
            }
        }
    }

    // Read the source file
    let source_code = fs::read_to_string(input_file)?;

    println!("Parsing source code from {}...", input_file);

    // Parse the program
    let ast = parse_program(source_code)?;

    println!("Compiling...");

    // Create LLVM context and codegen
    let context = Context::create();
    let mut codegen = CodeGen::new(&context)?;

    if let Some(out) = output_file {
        // Compile to executable file
        codegen.compile_to_executable(&ast, &out)?;
        println!("Wrote executable: {}", out);
        return Ok(());
    }

    // No output path requested: execute via JIT (preserve previous behavior)
    let result = codegen.execute_program(&ast)?;

    println!("Program executed successfully.");
    println!("Result: {}", result);

    Ok(())
}
