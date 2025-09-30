mod parser;
mod tokenizer;

use std::env::args;

use tokenizer::Lexer;

fn main() -> Result<(), String> {
    let input = args()
        .nth(1)
        .ok_or("Please provide an input string as a command line argument.")?;
    let mut lexer = Lexer::new(input);

    let tokens = lexer.tokenize()?;

    println!("{tokens:?}");

    Ok(())
}
