mod codegen;
mod parser;
mod tokenizer;

use codegen::CodeGen;
use inkwell::context::Context;
use parser::{parse_program, parse_program_verbose};
use std::env::args;
use std::fs;
use std::io::Write;

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
    let mut verbose = false;

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
            "--verbose" => {
                verbose = true;
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

    // Parse the program (with or without verbose mode)
    let (ast, tokens_opt) = if verbose {
        let (ast, tokens) = parse_program_verbose(source_code)?;
        (ast, Some(tokens))
    } else {
        (parse_program(source_code)?, None)
    };

    println!("Compiling...");

    // Create LLVM context and codegen
    let context = Context::create();
    let mut codegen = CodeGen::new(&context)?;

    // Compile to generate IR (needed for both execution and verbose output)
    let _ = codegen.compile_program(&ast)?;

    // If verbose mode is enabled, write debug info to file
    if verbose {
        let verbose_filename = format!("{}_verbose.txt", 
            input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output"));
        
        let mut verbose_file = fs::File::create(&verbose_filename)?;
        
        // Write tokens
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        writeln!(verbose_file, "TOKENS")?;
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        if let Some(tokens) = &tokens_opt {
            for (i, token) in tokens.iter().enumerate() {
                writeln!(verbose_file, "{:4}: {:?}", i + 1, token)?;
            }
        }
        writeln!(verbose_file)?;
        
        // Write AST
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        writeln!(verbose_file, "ABSTRACT SYNTAX TREE")?;
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        writeln!(verbose_file, "{:#?}", ast)?;
        writeln!(verbose_file)?;
        
        // Write LLVM IR
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        writeln!(verbose_file, "LLVM IR CODE")?;
        writeln!(verbose_file, "{}", "=".repeat(80))?;
        writeln!(verbose_file, "{}", codegen.get_ir_string())?;
        
        println!("Verbose output written to: {}", verbose_filename);
    }

    if let Some(out) = output_file {
        // Compile to executable file
        codegen.compile_to_executable(&ast, &out)?;
        println!("Wrote executable: {}", out);
        return Ok(());
    }

    // No output path requested: execute via JIT
    let result = codegen.execute_program(&ast)?;

    println!("Program executed successfully.");
    println!("Result: {}", result);

    Ok(())
}
