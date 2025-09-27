mod tokenizer;
mod parser;

use tokenizer::Lexer;

fn main() -> Result<(), String> {
    println!("MLIA Tokenizer Demo");
    println!("==================\n");

    // // Example 1: Simple integer literal
    // println!("Example 1: Integer literal");
    // println!("Input: '42'");
    // let mut lexer = Lexer::new("42".to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    // // Example 2: Negative integer
    // println!("Example 2: Negative integer");
    // println!("Input: '-123'");
    // let mut lexer = Lexer::new("-123".to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    // // Example 3: Identifier
    // println!("Example 3: Identifier");
    // println!("Input: 'hello_world'");
    // let mut lexer = Lexer::new("hello_world".to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    // // Example 4: Declaration statement
    // println!("Example 4: Declaration statement");
    // println!("Input: 'decl x <- 42'");
    // let mut lexer = Lexer::new("decl x <- 42".to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    // Example 5: Complex expression with whitespace 
    // println!("Example 5: Complex expression with whitespace");
    // let input = " decl   variable_name   <- match a with | 3 -> ( -999 ) (**)";
    // println!("Input: '{}'", input);
    // let mut lexer = Lexer::new(input.to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    // // Example 6: Multiple statements
    // println!("Example 6: Multiple statements");
    // println!("Input: 'decl a <- 1 ; decl b <- 2'");
    // let mut lexer = Lexer::new("decl a <- 1 ; decl b <- 2".to_string());
    // let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}\n", tokens);

    
    // Example 7: Full program
    println!("Example 7: Full program");
    // Valid example
    let src = include_str!("../docs/ejemplos.md");

    // Invalid example: invalid symbols
    //let src = include_str!("../docs/ejemploInv.md");
    // Strip the markdown code fences and header
    let mut lines = src.lines();
    // skip title
    let _ = lines.next();
    let mut collected = String::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            continue;
        }
        collected.push_str(line);
        collected.push('\n');
    }
    println!("Input: '{}'", collected);
    let mut lexer = Lexer::new(collected);
    let tokens = lexer.tokenize()?;
    println!("Tokens:");

    for token in &tokens {
        println!("{:?}", token);
    }

    Ok(())
}
