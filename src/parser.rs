pub(crate) use pomelo::pomelo;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Ident(String),
    Call(String, Vec<Expr>),
    Seq(Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>),
    Decl(String, Vec<String>, Box<Expr>, Box<Expr>),
}

pomelo! {
    %include {
        use crate::parser::Expr;
    }

    %token #[derive(Debug, Clone, PartialEq)] pub enum Token {};

    // Precedence rules (lowest to highest)
    // In should have LOWER precedence than Semicolon
    // so that In captures everything to the right including semicolons
    // %right In;
    // %left Semicolon;
    // %right Assign;

    %type IntegerLiteral i64;
    %type Identifier String;
    %type expr Expr;
    %type seq_expr Expr;
    %type atom_expr Expr;
    %type assign_expr Expr;
    %type call_expr Expr;
    %type program Expr;

    // Start symbol
    %start_symbol program;

    // Program is just an expression
    program ::= expr(e) { e }

    // Declaration expressions (lowest precedence - captures everything after In)
    expr ::= Decl Identifier(var) Assign assign_expr(val) In expr(body) {
        Expr::Decl(var, vec![], Box::new(val), Box::new(body))
    }
    expr ::= seq_expr(e) { e }

    // Sequence expressions
    seq_expr ::= seq_expr(first) Semicolon assign_expr(second) { Expr::Seq(Box::new(first), Box::new(second)) }
    seq_expr ::= assign_expr(e) { e }

    // Assignment expressions
    assign_expr ::= Identifier(var) Assign assign_expr(val) { Expr::Assign(var, Box::new(val)) }
    assign_expr ::= call_expr(e) { e }

    // Function call expressions
    call_expr ::= Identifier(func) atom_expr(arg) { Expr::Call(func, vec![arg]) }
    call_expr ::= Print atom_expr(arg) { Expr::Call("print".to_string(), vec![arg]) }
    call_expr ::= atom_expr(e) { e }

    // Atomic expressions (highest precedence)
    atom_expr ::= IntegerLiteral(n) { Expr::Number(n) }
    atom_expr ::= Identifier(id) { Expr::Ident(id) }
    atom_expr ::= ParenL expr(e) ParenR { e }

    // Dummy rules to ensure all token variants are generated
    // (These will never be matched due to precedence but will force generation)
    atom_expr ::= While { Expr::Ident("while".to_string()) }
    atom_expr ::= Do { Expr::Ident("do".to_string()) }
    atom_expr ::= Done { Expr::Ident("done".to_string()) }
    atom_expr ::= Match { Expr::Ident("match".to_string()) }
    atom_expr ::= With { Expr::Ident("with".to_string()) }
    atom_expr ::= Arrow { Expr::Ident("arrow".to_string()) }
    atom_expr ::= Pipe { Expr::Ident("pipe".to_string()) }
    atom_expr ::= Print { Expr::Ident("print".to_string()) }
    atom_expr ::= Equals { Expr::Ident("equals".to_string()) }
    atom_expr ::= NotEquals { Expr::Ident("notequals".to_string()) }
    atom_expr ::= Greater { Expr::Ident("greater".to_string()) }
    atom_expr ::= Less { Expr::Ident("less".to_string()) }
    atom_expr ::= Plus { Expr::Ident("plus".to_string()) }
    atom_expr ::= Minus { Expr::Ident("minus".to_string()) }
    atom_expr ::= Star { Expr::Ident("star".to_string()) }
    atom_expr ::= Slash { Expr::Ident("slash".to_string()) }
    atom_expr ::= Percent { Expr::Ident("percent".to_string()) }
    atom_expr ::= Eof { Expr::Ident("eof".to_string()) }
}

// Re-export the Token enum from the generated parser module
pub use parser::Token;

use crate::tokenizer::Lexer;

pub fn parse_program(input: String) -> Result<Expr, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new();

    for (i, token) in tokens.iter().enumerate() {
        if token == &Token::Eof {
            break;
        }
        println!("Feeding token {i}: {token:?}");
        if let Err(e) = parser.parse(token.clone()) {
            return Err(format!("Parse error at token {i}: {token:?}, error: {e:?}"));
        }
    }

    println!("Calling end_of_input");
    parser
        .end_of_input()
        .map_err(|e| format!("Parse error at end: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let result = parse_program("42".to_string());
        println!("Parse result: {result:?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_mlia_example() {
        let input = r"
            decl a <- 2 in
            decl b <- 3 in
            print b;
            print a;
            0
        ";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().expect("Tokenizing failed");
        println!("Generated tokens: {tokens:#?}");

        let result = parse_program(input.to_string());
        println!("Parse result for main.mlia: {result:?}");
        assert!(
            result.is_ok(),
            "Failed to parse main.mlia example: {result:?}"
        );
    }
}
