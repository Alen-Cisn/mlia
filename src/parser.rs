pub(crate) use pomelo::pomelo;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(i64),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Ident(String),
    Call(String, Vec<Expr>),
    Seq(Box<Expr>, Box<Expr>),
    Assign(String, Box<Expr>),
    Decl(String, Vec<String>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),            // (condition, body)
    Match(Box<Expr>, Vec<(Pattern, Expr)>), // (scrutinee, arms)
}

pomelo! {
    %include {
        use crate::parser::{Expr, Pattern};
    }

    %token #[derive(Debug, Clone, PartialEq)] pub enum Token {};

    // Precedence rules to resolve conflicts
    %right Semicolon;  // Right-associative to continue building sequences
    %left Assign;
    %left With;
    %left Identifier IntegerLiteral ParenL While Match;  // Atom tokens
    %right Pipe;
    %right In;

    %type IntegerLiteral i64;
    %type Identifier String;
    %type expr Expr;
    %type seq_expr Expr;
    %type atom_expr Expr;
    %type assign_expr Expr;
    %type call_expr Expr;
    %type program Expr;
    %type pattern Pattern;
    %type match_arms Vec<(Pattern, Expr)>;
    %type param_list Vec<String>;
    %type arg_list Vec<Expr>;

    // Start symbol
    %start_symbol program;

    // Program is just an expression
    program ::= expr(e) { e }

    // Declaration expressions (lowest precedence - captures everything after In)
    expr ::= Decl Identifier(var) Assign expr(val) In expr(body) {
        Expr::Decl(var, vec![], Box::new(val), Box::new(body))
    }
    expr ::= Decl Identifier(var) param_list(params) Assign expr(val) In expr(body) {
        Expr::Decl(var, params, Box::new(val), Box::new(body))
    }
    expr ::= seq_expr(e) { e }

    param_list ::= Identifier(param) { 
        vec![param]
    }
    param_list ::= param_list(mut list) Identifier(param) { 
        list.push(param); 
        list 
    }

    // Sequence expressions - make semicolon right-associative to avoid conflict
    // Allow any expr (including declarations) in sequences
    seq_expr ::= assign_expr(first) Semicolon expr(second) { Expr::Seq(Box::new(first), Box::new(second)) }
    seq_expr ::= assign_expr(e) [Semicolon] { e }

    // Assignment expressions
    assign_expr ::= Identifier(var) Assign assign_expr(val) { Expr::Assign(var, Box::new(val)) }
    assign_expr ::= call_expr(e) [Assign] { e }

    // Function call expressions - reorder to prefer call over plain identifier
    call_expr ::= Print atom_expr(arg) { Expr::Call("print".to_string(), vec![arg]) }
    call_expr ::= Plus atom_expr(arg1) atom_expr(arg2) { Expr::Call("+".to_string(), vec![arg1, arg2]) }
    call_expr ::= Minus atom_expr(arg1) atom_expr(arg2) { Expr::Call("-".to_string(), vec![arg1, arg2]) }
    call_expr ::= Star atom_expr(arg1) atom_expr(arg2) { Expr::Call("*".to_string(), vec![arg1, arg2]) }
    call_expr ::= Slash atom_expr(arg1) atom_expr(arg2) { Expr::Call("/".to_string(), vec![arg1, arg2]) }
    call_expr ::= Percent atom_expr(arg1) atom_expr(arg2) { Expr::Call("%".to_string(), vec![arg1, arg2]) }
    call_expr ::= Less atom_expr(arg1) atom_expr(arg2) { Expr::Call("<".to_string(), vec![arg1, arg2]) }
    call_expr ::= Greater atom_expr(arg1) atom_expr(arg2) { Expr::Call(">".to_string(), vec![arg1, arg2]) }
    call_expr ::= Equals atom_expr(arg1) atom_expr(arg2) { Expr::Call("=".to_string(), vec![arg1, arg2]) }
    call_expr ::= NotEquals atom_expr(arg1) atom_expr(arg2) { Expr::Call("!=".to_string(), vec![arg1, arg2]) }
    call_expr ::= Ampersand atom_expr(arg1) atom_expr(arg2) { Expr::Call("&".to_string(), vec![arg1, arg2]) }
    call_expr ::= Pipe atom_expr(arg1) atom_expr(arg2) { Expr::Call("|".to_string(), vec![arg1, arg2]) }
    call_expr ::= Exclam atom_expr(arg) { Expr::Call("!".to_string(), vec![arg]) }
    call_expr ::= atom_expr(e) { e }
    
    arg_list ::= atom_expr(arg) { 
        vec![arg]
    }
    arg_list ::= arg_list(mut list) atom_expr(arg) { 
        list.push(arg); 
        list 
    }

    // Atomic expressions (highest precedence)
    atom_expr ::= IntegerLiteral(n) { Expr::Number(n) }
    atom_expr ::= Identifier(id) { Expr::Ident(id) }
    atom_expr ::= ParenL Identifier(func) arg_list(args) ParenR { Expr::Call(func, args) }
    atom_expr ::= ParenL expr(e) ParenR { e }

    // While loop
    atom_expr ::= While expr(cond) Do expr(body) Done {
        Expr::While(Box::new(cond), Box::new(body))
    }

    // Match expression
    atom_expr ::= Match expr(scrutinee) With match_arms(arms) [With] {
        Expr::Match(Box::new(scrutinee), arms)
    }

    // Pattern rules
    pattern ::= IntegerLiteral(n) { Pattern::Literal(n) }
    pattern ::= Underscore { Pattern::Wildcard }

    // Match arms
    match_arms ::= Pipe pattern(p) Arrow expr(e) [Pipe] {
        vec![(p, e)]
    }
    match_arms ::= match_arms(mut arms) Pipe pattern(p) Arrow expr(e) [Pipe] {
        arms.push((p, e));
        arms
    }
}

// Re-export the Token enum from the generated parser module
pub use parser::Token;

/// Parse a complete MLIA program from source code string
pub fn parse_program(input: String) -> Result<Expr, String> {
    use crate::tokenizer::Lexer;
    
    // Tokenize the input
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| format!("Tokenization error: {}", e))?;
    
    // Parse the tokens
    let mut parser = parser::Parser::new();
    for token in tokens {
        parser.parse(token).map_err(|e| format!("Parse error: {:?}", e))?;
    }
    
    // Finish parsing and return the AST
    parser.end_of_input().map_err(|e| format!("Parse error at end of input: {:?}", e))
}

/// Parse program with verbose output: returns (AST, tokens)
pub fn parse_program_verbose(input: String) -> Result<(Expr, Vec<Token>), String> {
    use crate::tokenizer::Lexer;
    
    // Tokenize the input
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| format!("Tokenization error: {}", e))?;
    
    // Clone tokens for verbose output
    let tokens_for_output = tokens.clone();
    
    // Parse the tokens
    let mut parser = parser::Parser::new();
    for token in tokens {
        parser.parse(token).map_err(|e| format!("Parse error: {:?}", e))?;
    }
    
    // Finish parsing and return the AST with tokens
    let ast = parser.end_of_input().map_err(|e| format!("Parse error at end of input: {:?}", e))?;
    
    Ok((ast, tokens_for_output))
}

#[cfg(test)]
mod tests {
    use super::parser::*;
    use super::*;

    // T009: Parser tests for while loops
    #[test]
    fn test_while_loop_simple() {
        // Test: while x do print x done
        let mut parser = Parser::new();

        parser.parse(Token::While).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::Do).unwrap();
        parser.parse(Token::Print).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::Done).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "While loop should parse successfully");
        let expr = result.unwrap();

        match expr {
            Expr::While(cond, body) => {
                assert!(
                    matches!(*cond, Expr::Ident(ref s) if s == "x"),
                    "Condition should be identifier 'x'"
                );
                assert!(
                    matches!(*body, Expr::Call(ref f, _) if f == "print"),
                    "Body should be print call"
                );
            }
            _ => panic!("Expected While expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_while_loop_with_condition() {
        // Test: while 1 do 42 done
        let mut parser = Parser::new();

        parser.parse(Token::While).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Do).unwrap();
        parser.parse(Token::IntegerLiteral(42)).unwrap();
        parser.parse(Token::Done).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "While loop with literals should parse");
        let expr = result.unwrap();

        match expr {
            Expr::While(cond, body) => {
                assert!(matches!(*cond, Expr::Number(1)), "Condition should be 1");
                assert!(matches!(*body, Expr::Number(42)), "Body should be 42");
            }
            _ => panic!("Expected While expression"),
        }
    }

    #[test]
    fn test_nested_while_loops() {
        // Test: while x do while y do 1 done done
        let mut parser = Parser::new();

        parser.parse(Token::While).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::Do).unwrap();
        parser.parse(Token::While).unwrap();
        parser.parse(Token::Identifier("y".to_string())).unwrap();
        parser.parse(Token::Do).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Done).unwrap();
        parser.parse(Token::Done).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Nested while loops should parse");
        let expr = result.unwrap();

        match expr {
            Expr::While(_, body) => {
                assert!(
                    matches!(*body, Expr::While(_, _)),
                    "Body should be another while loop"
                );
            }
            _ => panic!("Expected outer While expression"),
        }
    }

    // T010: Parser tests for match expressions
    #[test]
    fn test_match_expression_simple() {
        // Test: match x with | 1 -> 10 | _ -> 20
        let mut parser = Parser::new();

        parser.parse(Token::Match).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::With).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(10)).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::Underscore).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(20)).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Match expression should parse successfully");
        let expr = result.unwrap();

        match expr {
            Expr::Match(scrutinee, arms) => {
                assert!(
                    matches!(*scrutinee, Expr::Ident(ref s) if s == "x"),
                    "Scrutinee should be 'x'"
                );
                assert_eq!(arms.len(), 2, "Should have 2 match arms");
                assert!(
                    matches!(arms[0].0, Pattern::Literal(1)),
                    "First pattern should be literal 1"
                );
                assert!(
                    matches!(arms[1].0, Pattern::Wildcard),
                    "Second pattern should be wildcard"
                );
            }
            _ => panic!("Expected Match expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_match_expression_multiple_literals() {
        // Test: match 5 with | 1 -> 10 | 2 -> 20 | 3 -> 30 | _ -> 0
        let mut parser = Parser::new();

        parser.parse(Token::Match).unwrap();
        parser.parse(Token::IntegerLiteral(5)).unwrap();
        parser.parse(Token::With).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(10)).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(2)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(20)).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(3)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(30)).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::Underscore).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(0)).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Match with multiple arms should parse");
        let expr = result.unwrap();

        match expr {
            Expr::Match(_, arms) => {
                assert_eq!(arms.len(), 4, "Should have 4 match arms");
                assert!(matches!(arms[0].0, Pattern::Literal(1)));
                assert!(matches!(arms[1].0, Pattern::Literal(2)));
                assert!(matches!(arms[2].0, Pattern::Literal(3)));
                assert!(matches!(arms[3].0, Pattern::Wildcard));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_match_with_wildcard_only() {
        // Test: match x with | _ -> 42
        let mut parser = Parser::new();

        parser.parse(Token::Match).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::With).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::Underscore).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(42)).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Match with only wildcard should parse");
        let expr = result.unwrap();

        match expr {
            Expr::Match(_, arms) => {
                assert_eq!(arms.len(), 1, "Should have 1 match arm");
                assert!(
                    matches!(arms[0].0, Pattern::Wildcard),
                    "Pattern should be wildcard"
                );
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_match_expression_with_complex_result() {
        // Test: match x with | 1 -> print x | _ -> 0
        let mut parser = Parser::new();

        parser.parse(Token::Match).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::With).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::Print).unwrap();
        parser.parse(Token::Identifier("x".to_string())).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::Underscore).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(0)).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Match with expression results should parse");
        let expr = result.unwrap();

        match expr {
            Expr::Match(_, arms) => {
                assert_eq!(arms.len(), 2);
                assert!(
                    matches!(arms[0].1, Expr::Call(ref f, _) if f == "print"),
                    "First result should be print call"
                );
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_pattern_literal() {
        // Test that literal patterns parse correctly
        let mut parser = Parser::new();

        parser.parse(Token::Match).unwrap();
        parser.parse(Token::IntegerLiteral(100)).unwrap();
        parser.parse(Token::With).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::IntegerLiteral(100)).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(1)).unwrap();
        parser.parse(Token::Pipe).unwrap();
        parser.parse(Token::Underscore).unwrap();
        parser.parse(Token::Arrow).unwrap();
        parser.parse(Token::IntegerLiteral(0)).unwrap();
        let result = parser.end_of_input();

        assert!(result.is_ok(), "Match with literal pattern should parse");
    }
}
