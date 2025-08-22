use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    IntegerLiteral(i64),
    Identifier(String),

    Decl,

    Equals,

    Eof,
}

// Este es el lexer.
// input es el valor que entra y que va a ser convertido en tokens.
// position es la posición actual del cursor, los anteriores ya fueron leidos.
// line va aumentando a medida que se leen saltos de linea.
// column va aumentando a medida que se leen caracteres y se resetea a 1 cuando se lee un salto de linea.

pub enum State {
    Start,
    Digit,
    LetterOrUnderscore,
    Equals,
    Minus,
    Eof,
}


#[derive(Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.position < self.input.chars().count() {
            self.skip_whitespace();

            if self.position >= self.input.chars().count() {
                break;
            }
            if let Ok(token) = self.next_token() {
                tokens.push(token);
            } else {
                let error_message = self.next_token().err().unwrap();
                return Err(format!("Error al tokenizar en la línea {}, columna {}: {}", self.line, self.column, error_message));
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn is_letter_or_underscore(c: char) -> bool {
        match c {
            'a'..='z'
            | 'A'..='Z'
            | '\u{00DF}'..='\u{00F6}'
            | '\u{00F8}'..='\u{00FF}'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00DE}'
            | '\u{0153}'
            | '\u{0161}'
            | '\u{017E}'
            | '\u{017D}'
            | '\u{0152}'
            | '\u{0160}'
            | '\u{0178}'
            | '\u{1E9E}'
            | '_'
            => true,
            _ => false,
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let current_char = self.current_char()?;

        match current_char {
            '0'..='9' => self.tokenize_integer_literal(),
            _ if Self::is_letter_or_underscore(current_char) => self.tokenize_identifier_or_keyword(),

            '=' => {
                self.advance();
                Ok(Token::Equals)
            }

            '-' => {
                if self.peek_next_char()?.map_or(false, |c| c.is_ascii_digit()) {
                    self.tokenize_integer_literal()
                } else {
                    Err(format!(
                        "Caracter inesperado '{}' en la línea {}, columna {}",
                        current_char, self.line, self.column
                    ))
                }
            }

            _ => Err(format!(
                "Caracter inesperado '{}' en la línea {}, columna {}",
                current_char, self.line, self.column
            )),
        }
    }

    fn tokenize_integer_literal(&mut self) -> Result<Token, String> {
        let mut value = String::new();

        if self.current_char()? == '-' {
            value.push('-');
            self.advance();
        }

        while self.position < self.input.chars().count() {
            let c = self.current_char()?;
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
            } else if c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        match value.parse::<i64>() {
            Ok(num) => Ok(Token::IntegerLiteral(num)),
            Err(_) => Err(format!(
                "Error al parsear el entero '{}' en la línea {}, columna {}",
                value, self.line, self.column
            )),
        }
    }

    fn tokenize_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut identifier = String::new();

        while self.position < self.input.chars().count() {
            let c = self.current_char()?;
            if Self::is_letter_or_underscore(c) || c.is_ascii_digit() {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match identifier.as_str() {
            "decl" => Ok(Token::Decl),
            _ => Ok(Token::Identifier(identifier)),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.chars().count() {
            let c = self.current_char().unwrap_or(' ');
            // Los "whitespace" son los caracteres que no se imprimen en la consola.
            // usamos el método std de Rust, que usa la definición de Unicode.
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    // Luego advance() lo posiciona en 1.
                    self.column = 0;
                }
                self.advance();
            } else {
                // Si no es un "whitespace", salimos.
                break;
            }
        }
    }

    fn current_char(&self) -> Result<char, String> {
        if self.position >= self.input.chars().count() {
            Err("Fin de la entrada inesperado".to_string())
        } else {
            let result = self.input.chars().nth(self.position);
            if let Some(c) = result {
                Ok(c)
            } else {
                let error_message = format!("Fin de la entrada inesperado en la línea {}, columna {}:", self.line, self.column);
                Err(error_message)
            }
        }
    }

    fn peek_next_char(&self) -> Result<Option<char>, String> {
        if self.position + 1 >= self.input.chars().count() {
            Ok(None)
        } else {
            Ok(Some(self.input.chars().nth(self.position + 1).unwrap()))
        }
    }
    fn advance(&mut self) {
        if self.position < self.input.chars().count() {
            self.position += 1;
            self.column += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_literals() {
        let mut lexer = Lexer::new("123 -456_123 0".to_string());
        let tokens = lexer.tokenize();
        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::IntegerLiteral(123), "El token 0 no es un entero: {:?}", tokens[0]);
        assert_eq!(tokens[1], Token::IntegerLiteral(-456123), "El token 1 no es un entero: {:?}", tokens[1]);
        assert_eq!(tokens[2], Token::IntegerLiteral(0), "El token 2 no es un entero: {:?}", tokens[2]);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("hola mundo cómo estas _test".to_string());
        let tokens = lexer.tokenize();

        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::Identifier("hola".to_string()));
        assert_eq!(tokens[1], Token::Identifier("mundo".to_string()));
        assert_eq!(tokens[2], Token::Identifier("cómo".to_string()));
        assert_eq!(tokens[3], Token::Identifier("estas".to_string()));
        assert_eq!(tokens[4], Token::Identifier("_test".to_string()));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("decl".to_string());
        let tokens = lexer.tokenize();
        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::Decl, "El token 0 no es un identificador: {:?}", tokens[0]);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("=".to_string());
        let tokens = lexer.tokenize();
        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::Equals, "El token 0 no es un operador: {:?}", tokens[0]);
    }

    #[test]
    fn test_declaration() {
        let mut lexer = Lexer::new("decl x = 42".to_string());
        let tokens = lexer.tokenize();
        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::Decl, "El token 0 no es un identificador: {:?}", tokens[0]);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()), "El token 1 no es un identificador: {:?}", tokens[1]);
        assert_eq!(tokens[2], Token::Equals, "El token 2 no es un operador: {:?}", tokens[2]);
        assert_eq!(tokens[3], Token::IntegerLiteral(42), "El token 3 no es un entero: {:?}", tokens[3]);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  decl   x   =   123  \n  decl   y   =   456  ".to_string());
        let tokens = lexer.tokenize();
        assert!(!tokens.is_err(), "El lexer no debería devolver un error: {:?}", tokens);
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::Decl, "El token 0 no es un identificador: {:?}", tokens[0]);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()), "El token 1 no es un identificador: {:?}", tokens[1]);
        assert_eq!(tokens[2], Token::Equals, "El token 2 no es un operador: {:?}", tokens[2]);
        assert_eq!(tokens[3], Token::IntegerLiteral(123), "El token 3 no es un entero: {:?}", tokens[3]);
        assert_eq!(tokens[4], Token::Decl, "El token 4 no es un identificador: {:?}", tokens[4]);
        assert_eq!(tokens[5], Token::Identifier("y".to_string()), "El token 5 no es un identificador: {:?}", tokens[5]);
        assert_eq!(tokens[6], Token::Equals, "El token 6 no es un operador: {:?}", tokens[6]);
        assert_eq!(tokens[7], Token::IntegerLiteral(456), "El token 7 no es un entero: {:?}", tokens[7]);
    }
}
