// use pomelo::pomelo;

// use crate::tokenizer::Token;

   
// pub enum Expr {
//     Number(i64),
//     Ident(String),
//     Call(String, Vec<Expr>),
//     Seq(Box<Expr>, Box<Expr>),
//     Assign(String, Box<Expr>),
//     Decl(String, Vec<String>, Box<Expr>, Box<Expr>)
// } 



// pomelo! {
     
//     %token #[derive(Debug)] pub enum Token {};
//    // %extra_argument Program;
//     %type Number i64;
//     %type Ident String;
//     %type expr Expr;
 
//     expr ::= Number(n) { Expr::Number(n) }
//     expr ::= Ident(id) { Expr::Ident(id) }

// }