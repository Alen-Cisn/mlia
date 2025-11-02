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
    let mut _output_file: Option<String> = None;

    // Parse command line arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    _output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a filename".into());
                }
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

    println!("Compiling to LLVM IR...");

    // Create LLVM context and codegen
    let context = Context::create();
    let mut codegen = CodeGen::new(&context)?;

    // Execute the program and get result
    let result = codegen.execute_program(&ast)?;

    println!("Program executed successfully.");
    println!("Result: {}", result);

    Ok(())
}
