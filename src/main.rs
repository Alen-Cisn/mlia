mod codegen;
mod parser;
mod tokenizer;

use std::env::args;
use std::fs;

use codegen::CodeGen;
use inkwell::context::Context;
use parser::parse_program;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        return Err("Please provide an input file as a command line argument.".into());
    }

    let input_file = &args[1];
    let mut output_file: Option<String> = None;
    let mut jit_mode = false;

    // Parse command line arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    jit_mode = false;
                    i += 2;
                } else {
                    return Err("--output requires a filename".into());
                }
            }
            "--exe" => {
                // Generate executable with same name as input but .exe extension
                let base_name = input_file.strip_suffix(".mlia").unwrap_or(input_file);
                output_file = Some(format!("{}.exe", base_name));
                jit_mode = false;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]).into());
            }
        }
    }

    // Read the file content
    let source_code = if input_file.ends_with(".mlia") {
        fs::read_to_string(input_file)?
    } else {
        // If it's not a file, treat it as direct source code
        input_file.clone()
    };

    println!("Parsing source code...");

    // Parse the source code into an AST
    let ast = parse_program(source_code)?;
    println!("Parse result: {ast:?}");

    // Create LLVM context and code generator
    let context = Context::create();
    let mut codegen = CodeGen::new(&context)?;

    if jit_mode {
        println!("\nCompiling and executing with JIT...");

        // Execute the program with JIT
        let result = codegen.execute_program(&ast)?;

        // Print the generated LLVM IR for debugging
        println!("\nGenerated LLVM IR:");
        codegen.print_ir();

        println!("Program returned: {}", result);
    } else {
        println!("\nCompiling to executable...");

        let output_path =
            output_file.unwrap_or_else(|| input_file.trim_end_matches(".mlia").to_string());

        codegen.compile_to_executable(&ast, &output_path)?;

        // Print the generated LLVM IR for debugging
        println!("\nGenerated LLVM IR:");
        codegen.print_ir();
    }

    Ok(())
}
