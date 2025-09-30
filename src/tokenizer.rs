use crate::parser::Token;
use std::collections::HashMap;

// Este es el lexer.
// input es el valor que entra y que va a ser convertido en tokens.
// position es la posición actual del cursor, los anteriores ya fueron leidos.
// line va aumentando a medida que se leen saltos de linea.
// column va aumentando a medida que se leen caracteres y se resetea a 1 cuando se lee un salto de linea.

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Start = 0,                             // q0
    Digit = 1,                             // q1
    PipeOrIdentifier = 2,                  // q2
    AssignOrIdentifier = 3,                // q3
    FinishAssignOrIdentifier = 4,          // q4
    Identifier = 5,                        // q5
    FinishArrowOrIdentifier = 6,           // q6
    ArrowOrIdentifierOrNegativeNumber = 7, // q7
    ParenLOrComment = 8,                   // q8
    Comment = 9,                           // q9
    MayFinishComment = 10,                 // q10
    ParenR = 11,                           // q11
}

impl State {
    pub const COUNT: usize = 12;
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Start),
            1 => Some(Self::Digit),
            2 => Some(Self::PipeOrIdentifier),
            3 => Some(Self::AssignOrIdentifier),
            4 => Some(Self::FinishAssignOrIdentifier),
            5 => Some(Self::Identifier),
            6 => Some(Self::FinishArrowOrIdentifier),
            7 => Some(Self::ArrowOrIdentifierOrNegativeNumber),
            8 => Some(Self::ParenLOrComment),
            9 => Some(Self::Comment),
            10 => Some(Self::MayFinishComment),
            11 => Some(Self::ParenR),
            _ => None,
        }
    }
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CharClass {
    Digit = 0,       // 0..9
    LowerAlpha = 1,  // a..z
    UpperAlpha = 2,  // A..Z
    Less = 3,        // <
    Greater = 4,     // >
    Minus = 5,       // -
    Plus = 6,        // +
    Star = 7,        // *
    Slash = 8,       // /
    Equals = 9,      // =
    Exclam = 10,     // !
    Percent = 11,    // %
    Caret = 12,      // ^
    Underscore = 13, // _
    Pipe = 14,       // |
    LParen = 15,     // (
    RParen = 16,     // )
    Semicolon = 17,  // ;
    Whitespace = 18, // whitespace (including CR, LF, TAB)
    PunctGroup = 19, // {, }, [, ], ., :
}

impl CharClass {
    pub const COUNT: usize = 20;
}

pub const fn classify_char(c: char) -> Option<CharClass> {
    use CharClass::{
        Caret, Digit, Equals, Exclam, Greater, LParen, Less, LowerAlpha, Minus, Percent, Pipe,
        Plus, PunctGroup, RParen, Semicolon, Slash, Star, Underscore, UpperAlpha, Whitespace,
    };
    match c {
        '0'..='9' => Some(Digit),
        'a'..='z' | '\u{00DF}'..='\u{00F6}' | '\u{00F8}'..='\u{00FF}' => Some(LowerAlpha),
        'A'..='Z' | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00DE}' => Some(UpperAlpha),
        '<' => Some(Less),
        '>' => Some(Greater),
        '-' => Some(Minus),
        '+' => Some(Plus),
        '*' => Some(Star),
        '/' => Some(Slash),
        '=' => Some(Equals),
        '!' => Some(Exclam),
        '%' => Some(Percent),
        '^' => Some(Caret),
        '_' => Some(Underscore),
        '|' => Some(Pipe),
        '(' => Some(LParen),
        ')' => Some(RParen),
        ';' => Some(Semicolon),
        '{' | '}' | '[' | ']' | '.' | ':' => Some(PunctGroup),
        _ if c.is_whitespace() => Some(Whitespace),
        _ => None,
    }
}

pub const fn is_identifier_char(c: char) -> bool {
    match c {
        '0'..='9'
        | 'a'..='z'
        | '\u{00DF}'..='\u{00F6}'
        | '\u{00F8}'..='\u{00FF}'
        | 'A'..='Z'
        | '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00DE}'
        | '<'
        | '>'
        | '-'
        | '+'
        | '*'
        | '/'
        | '='
        | '!'
        | '%'
        | '^'
        | '_'
        | '|' => true,
        '(' | ')' | ';' | '{' | '}' | '[' | ']' | '.' | ':' => false,
        _ if c.is_whitespace() => false,
        _ => false,
    }
}

pub const NUM_STATES: usize = State::COUNT;
pub const NUM_CLASSES: usize = CharClass::COUNT;

// -1 means no valid transition from that state with that char class
pub const STATE_TRANSITIONS: [[i8; NUM_CLASSES]; NUM_STATES] = [
    // q0 (Start)
    [1, 5, 3, 3, 7, 7, 5, 5, 5, 5, 5, 5, 5, 2, 2, 8, 11, 0, 0, -1],
    // q1 (Digit)
    [
        1, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -1, -1, -1, -1, -1, -1,
    ],
    // q2 (PipeOrIdentifier)
    [
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q3 (AssignOrIdentifier)
    [
        5, 5, 5, 5, 5, 4, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q4 (FinishAssignOrIdentifier)
    [
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q5 (Identifier)
    [
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q6 (FinishArrowOrIdentifier)
    [
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q7 (ArrowOrIdentifier)
    [
        1, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, -1, -1, -1, -1, -1, -1,
    ],
    // q8 (ParenLOrComment)
    [
        -1, -1, -1, -1, -1, -1, -1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ],
    // q9 (Comment)
    [9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, -1],
    // q10 (MayFinishComment)
    [9, 9, 9, 9, 9, 9, 9, 10, 9, 9, 9, 9, 9, 9, 9, 9, 0, 9, 9, -1],
    // q11 (ParenR)
    [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ],
];

pub fn next_state(current: State, class: CharClass) -> Result<Option<State>, String> {
    let idx = STATE_TRANSITIONS[current as usize][class as usize];
    if idx == -1 {
        Ok(None)
    } else if idx == -2 {
        Err("Error, caracter inválido".to_string())
    } else {
        Ok(State::from_index(idx as usize))
    }
}

pub static KEYWORDS: std::sync::LazyLock<HashMap<&'static str, Token>> =
    std::sync::LazyLock::new(|| {
        const KEYWORDS: &[(&str, Token)] = &[
            ("decl", Token::Decl),
            ("while", Token::While),
            ("do", Token::Do),
            ("done", Token::Done),
            ("match", Token::Match),
            ("with", Token::With),
            ("in", Token::In),
            // funciones built-in
            ("print", Token::Print),
            ("<", Token::Less),
            (">", Token::Greater),
            ("+", Token::Plus),
            ("-", Token::Minus),
            ("*", Token::Star),
            ("/", Token::Slash),
            ("%", Token::Percent),
            ("=", Token::Equals),
            ("!=", Token::NotEquals),
            ("->", Token::Arrow),
            ("<-", Token::Assign),
            ("|", Token::Pipe),
            (";", Token::Semicolon),
            ("(", Token::ParenL),
            (")", Token::ParenR),
        ];
        let mut m = HashMap::new();
        for &(k, ref v) in KEYWORDS {
            m.insert(k, v.clone());
        }
        m
    });

#[derive(Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
    current_lexeme: String,
    tokens: Vec<Token>,
}

impl Lexer {
    pub const fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            current_lexeme: String::new(),
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        self.tokens.clear();
        self.current_lexeme.clear();

        let chars: Vec<char> = self.input.chars().collect();
        let mut index: usize = 0;
        let mut state = State::Start;

        while index < chars.len() {
            let c = chars[index];
            let next_ch = if index + 1 < chars.len() {
                Some(chars[index + 1])
            } else {
                None
            };
            let Some(class) = classify_char(c) else {
                return Err(format!(
                    "Caracter inesperado '{}' en la línea {}, columna {}",
                    c, self.line, self.column
                ));
            };
            println!("Estado: {state:?}, Char: '{c}', Clase: {class:?} -> ");

            let next = next_state(state, class);

            if let Err(e) = next {
                return Err(format!(
                    "{} '{}' en la línea {}, columna {}",
                    e, c, self.line, self.column
                ));
            } else if let Ok(Some(next_state_value)) = next {
                // Execute transition action
                let action = TRANSITION_ACTIONS[state as usize][class as usize];
                (action)(self, Some(c), next_ch);

                // Advance position and line/column
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                self.column += 1;
                index += 1;
                self.position = index;

                // Move to next state
                state = next_state_value;
            } else {
                // No transition: finalize current lexeme if any (do not consume current char)
                self.finalize_lexeme(state)?;
                state = State::Start;
                // Note: Do not advance index; reprocess this char from Start
            }
        }

        // End of input: finalize any pending lexeme
        self.finalize_lexeme(state)?;
        self.tokens.push(Token::Eof);
        Ok(std::mem::take(&mut self.tokens))
    }

    fn append_char(&mut self, c: char) {
        self.current_lexeme.push(c);
    }

    fn clear_lexeme(&mut self) {
        self.current_lexeme.clear();
    }

    fn finalize_lexeme(&mut self, state: State) -> Result<(), String> {
        if self.current_lexeme.is_empty() {
            return Ok(());
        }

        match state {
            State::Digit => {
                // Integer literal
                let parsed = self.current_lexeme.parse::<i64>().map_err(|_| {
                    format!(
                        "Error al parsear el entero '{}' en la línea {}, columna {}",
                        self.current_lexeme, self.line, self.column
                    )
                })?;
                self.tokens.push(Token::IntegerLiteral(parsed));
                self.clear_lexeme();
                Ok(())
            }
            State::PipeOrIdentifier
            | State::AssignOrIdentifier
            | State::FinishAssignOrIdentifier
            | State::Identifier
            | State::FinishArrowOrIdentifier
            | State::ArrowOrIdentifierOrNegativeNumber => {
                // Identifier or keyword
                if let Some(keyword_token) = KEYWORDS.get(self.current_lexeme.as_str()) {
                    self.tokens.push(keyword_token.clone());
                } else {
                    self.tokens
                        .push(Token::Identifier(std::mem::take(&mut self.current_lexeme)));
                }
                self.clear_lexeme();
                Ok(())
            }
            _ => {
                // Nothing to finalize
                self.clear_lexeme();
                Ok(())
            }
        }
    }
}

pub type TransitionAction = fn(&mut Lexer, Option<char>, Option<char>);

const fn action_noop(_: &mut Lexer, _: Option<char>, _: Option<char>) {}
fn action_start_lexeme(lexer: &mut Lexer, ch: Option<char>, _next_ch: Option<char>) {
    if let Some(c) = ch {
        lexer.append_char(c);
    }
}
fn action_append_lexeme(lexer: &mut Lexer, ch: Option<char>, _next_ch: Option<char>) {
    if let Some(c) = ch {
        lexer.append_char(c);
    }
}
fn action_emit_semicolon(lexer: &mut Lexer, _: Option<char>, _next_ch: Option<char>) {
    lexer.tokens.push(Token::Semicolon);
    lexer.clear_lexeme();
}

fn action_emit_pipe(lexer: &mut Lexer, _: Option<char>, _next_ch: Option<char>) {
    lexer.tokens.push(Token::Pipe);
    lexer.clear_lexeme();
}

fn action_maybe_emit_assign(lexer: &mut Lexer, _: Option<char>, next_ch: Option<char>) {
    if lexer.current_lexeme.as_str() == "<-" && !is_identifier_char(next_ch.unwrap_or(' ')) {
        lexer.tokens.push(Token::Assign);
        lexer.clear_lexeme();
    }
}

fn action_maybe_emit_arrow(lexer: &mut Lexer, _: Option<char>, next_ch: Option<char>) {
    if lexer.current_lexeme.as_str() == "->" && !is_identifier_char(next_ch.unwrap_or(' ')) {
        lexer.tokens.push(Token::Arrow);
        lexer.clear_lexeme();
    }
}

fn action_append_and_maybe_emit_assign(lexer: &mut Lexer, ch: Option<char>, next_ch: Option<char>) {
    if let Some(c) = ch {
        lexer.append_char(c);
    }
    action_maybe_emit_assign(lexer, None, next_ch);
}

fn action_append_and_maybe_emit_arrow(lexer: &mut Lexer, ch: Option<char>, next_ch: Option<char>) {
    if let Some(c) = ch {
        lexer.append_char(c);
    }
    action_maybe_emit_arrow(lexer, None, next_ch);
}

fn action_maybe_emit_paren_l(lexer: &mut Lexer, _: Option<char>, next_ch: Option<char>) {
    // Check if the next character is '*' to start a comment, otherwise emit ParenL
    if next_ch != Some('*') {
        lexer.tokens.push(Token::ParenL);
    }
    lexer.clear_lexeme();
}

fn action_maybe_emit_paren_r(lexer: &mut Lexer, _: Option<char>, _: Option<char>) {
    lexer.tokens.push(Token::ParenR);
    lexer.clear_lexeme();
}

fn action_clear_paren_l(lexer: &mut Lexer, _: Option<char>, _: Option<char>) {
    // Clear any accumulated characters when starting a comment
    lexer.clear_lexeme();
}

fn action_end_comment(lexer: &mut Lexer, _: Option<char>, _: Option<char>) {
    // End comment and clear lexeme, transition back to Start will be handled by state machine
    lexer.clear_lexeme();
}

// Transition actions per [State][CharClass]
pub static TRANSITION_ACTIONS: [[TransitionAction; NUM_CLASSES]; NUM_STATES] = [
    // q0 (Start)
    [
        action_start_lexeme,       // Digit
        action_start_lexeme,       // LowerAlpha
        action_start_lexeme,       // UpperAlpha
        action_start_lexeme,       // < (may start identifier or <-)
        action_start_lexeme,       // >
        action_start_lexeme,       // - (may start integer or -> or identifier)
        action_start_lexeme,       // +
        action_start_lexeme,       // *
        action_start_lexeme,       // /
        action_start_lexeme,       // =
        action_start_lexeme,       // !
        action_start_lexeme,       // %
        action_start_lexeme,       // ^
        action_start_lexeme,       // _
        action_start_lexeme,       // | (  start identifier too)
        action_maybe_emit_paren_l, // (
        action_maybe_emit_paren_r, // )
        action_emit_semicolon,     // ;
        action_noop,               // whitespace
        action_noop,               // { } [ ] . :
    ],
    // q1 (Digit)
    [
        action_append_lexeme, // Digit
        action_noop,          // LowerAlpha
        action_noop,          // UpperAlpha
        action_noop,          // <
        action_noop,          // >
        action_noop,          // -
        action_noop,          // +
        action_noop,          // *
        action_noop,          // /
        action_noop,          // =
        action_noop,          // !
        action_noop,          // %
        action_noop,          // ^
        action_noop,          // _
        action_noop,          // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q2 (PipeOrIdentifier)
    [
        action_append_lexeme, // Digit
        action_append_lexeme, // LowerAlpha
        action_append_lexeme, // UpperAlpha
        action_append_lexeme, // <
        action_append_lexeme, // >
        action_append_lexeme, // -
        action_append_lexeme, // +
        action_append_lexeme, // *
        action_append_lexeme, // /
        action_append_lexeme, // =
        action_append_lexeme, // !
        action_append_lexeme, // %
        action_append_lexeme, // ^
        action_append_lexeme, // _
        action_emit_pipe,     // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q3 (AssignOrIdentifier)
    [
        action_append_lexeme,                // Digit
        action_append_lexeme,                // LowerAlpha
        action_append_lexeme,                // UpperAlpha
        action_append_lexeme,                // <
        action_append_and_maybe_emit_assign, // > (complete <-)
        action_append_lexeme,                // -
        action_append_lexeme,                // +
        action_append_lexeme,                // *
        action_append_lexeme,                // /
        action_append_lexeme,                // =
        action_append_lexeme,                // !
        action_append_lexeme,                // %
        action_append_lexeme,                // ^
        action_append_lexeme,                // _
        action_noop,                         // |
        action_noop,                         // (
        action_noop,                         // )
        action_noop,                         // ;
        action_noop,                         // whitespace
        action_noop,                         // punct group
    ],
    // q4 (FinishAssignOrIdentifier)
    [
        action_append_lexeme, // Digit
        action_append_lexeme, // LowerAlpha
        action_append_lexeme, // UpperAlpha
        action_append_lexeme, // <
        action_append_lexeme, // >
        action_append_lexeme, // -
        action_append_lexeme, // +
        action_append_lexeme, // *
        action_append_lexeme, // /
        action_append_lexeme, // =
        action_append_lexeme, // !
        action_append_lexeme, // %
        action_append_lexeme, // ^
        action_append_lexeme, // _
        action_noop,          // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q5 (Identifier)
    [
        action_append_lexeme, // Digit
        action_append_lexeme, // LowerAlpha
        action_append_lexeme, // UpperAlpha
        action_append_lexeme, // <
        action_append_lexeme, // >
        action_append_lexeme, // -
        action_append_lexeme, // +
        action_append_lexeme, // *
        action_append_lexeme, // /
        action_append_lexeme, // =
        action_append_lexeme, // !
        action_append_lexeme, // %
        action_append_lexeme, // ^
        action_append_lexeme, // _
        action_noop,          // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q6 (FinishArrowOrIdentifier)
    [
        action_append_lexeme, // Digit
        action_append_lexeme, // LowerAlpha
        action_append_lexeme, // UpperAlpha
        action_append_lexeme, // <
        action_append_lexeme, // >
        action_append_lexeme, // -
        action_append_lexeme, // +
        action_append_lexeme, // *
        action_append_lexeme, // /
        action_append_lexeme, // =
        action_append_lexeme, // !
        action_append_lexeme, // %
        action_append_lexeme, // ^
        action_append_lexeme, // _
        action_noop,          // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q7 (ArrowOrIdentifier)
    [
        action_append_lexeme,               // Digit
        action_append_lexeme,               // LowerAlpha
        action_append_lexeme,               // UpperAlpha
        action_append_lexeme,               // <
        action_append_and_maybe_emit_arrow, // > (complete ->)
        action_append_lexeme,               // -
        action_append_lexeme,               // +
        action_append_lexeme,               // *
        action_append_lexeme,               // /
        action_append_lexeme,               // =
        action_append_lexeme,               // !
        action_append_lexeme,               // %
        action_append_lexeme,               // ^
        action_append_lexeme,               // _
        action_noop,                        // |
        action_noop,                        // (
        action_noop,                        // )
        action_noop,                        // ;
        action_noop,                        // whitespace
        action_noop,                        // punct group
    ],
    // q8 (ParenLOrComment)
    [
        action_noop,          // Digit
        action_noop,          // LowerAlpha
        action_noop,          // UpperAlpha
        action_noop,          // <
        action_noop,          // >
        action_noop,          // -
        action_noop,          // +
        action_clear_paren_l, // *
        action_noop,          // /
        action_noop,          // =
        action_noop,          // !
        action_noop,          // %
        action_noop,          // ^
        action_noop,          // _
        action_noop,          // |
        action_noop,          // (
        action_noop,          // )
        action_noop,          // ;
        action_noop,          // whitespace
        action_noop,          // punct group
    ],
    // q9 (Comment)
    [
        action_noop, // Digit
        action_noop, // LowerAlpha
        action_noop, // UpperAlpha
        action_noop, // <
        action_noop, // >
        action_noop, // -
        action_noop, // +
        action_noop, // *
        action_noop, // /
        action_noop, // =
        action_noop, // !
        action_noop, // %
        action_noop, // ^
        action_noop, // _
        action_noop, // |
        action_noop, // (
        action_noop, // )
        action_noop, // ;
        action_noop, // whitespace
        action_noop, // punct group
    ],
    // q10 (MayFinishComment)
    [
        action_noop,        // Digit
        action_noop,        // LowerAlpha
        action_noop,        // UpperAlpha
        action_noop,        // <
        action_noop,        // >
        action_noop,        // -
        action_noop,        // +
        action_noop,        // *
        action_noop,        // /
        action_noop,        // =
        action_noop,        // !
        action_noop,        // %
        action_noop,        // ^
        action_noop,        // _
        action_noop,        // |
        action_noop,        // (
        action_end_comment, // ) - end the comment
        action_noop,        // ;
        action_noop,        // whitespace
        action_noop,        // punct group
    ],
    // q11 (ParenR)
    [
        action_noop, // Digit
        action_noop, // LowerAlpha
        action_noop, // UpperAlpha
        action_noop, // <
        action_noop, // >
        action_noop, // -
        action_noop, // +
        action_noop, // *
        action_noop, // /
        action_noop, // =
        action_noop, // !
        action_noop, // %
        action_noop, // ^
        action_noop, // _
        action_noop, // |
        action_noop, // (
        action_noop, // )
        action_noop, // ;
        action_noop, // whitespace
        action_noop, // punct group
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_literals() {
        let mut lexer = Lexer::new("123 456123 0".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::IntegerLiteral(123),
            "El token 0 no es un entero: {:?}",
            tokens[0]
        );
        assert_eq!(
            tokens[1],
            Token::IntegerLiteral(456_123),
            "El token 1 no es un entero: {:?}",
            tokens[1]
        );
        assert_eq!(
            tokens[2],
            Token::IntegerLiteral(0),
            "El token 2 no es un entero: {:?}",
            tokens[2]
        );
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("hola mundo cómo estas _test".to_string());
        let tokens = lexer.tokenize();

        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
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
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::Decl,
            "El token 0 no es un identificador: {:?}",
            tokens[0]
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("=".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::Equals,
            "El token 0 no es un operador: {:?}",
            tokens[0]
        );
    }

    #[test]
    fn test_declaration() {
        let mut lexer = Lexer::new("decl x = 42".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::Decl,
            "El token 0 no es un identificador: {:?}",
            tokens[0]
        );
        assert_eq!(
            tokens[1],
            Token::Identifier("x".to_string()),
            "El token 1 no es un identificador: {:?}",
            tokens[1]
        );
        assert_eq!(
            tokens[2],
            Token::Equals,
            "El token 2 no es un operador: {:?}",
            tokens[2]
        );
        assert_eq!(
            tokens[3],
            Token::IntegerLiteral(42),
            "El token 3 no es un entero: {:?}",
            tokens[3]
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  decl   x   =   123  \n  decl   y   =   456  ".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::Decl,
            "El token 0 no es un identificador: {:?}",
            tokens[0]
        );
        assert_eq!(
            tokens[1],
            Token::Identifier("x".to_string()),
            "El token 1 no es un identificador: {:?}",
            tokens[1]
        );
        assert_eq!(
            tokens[2],
            Token::Equals,
            "El token 2 no es un operador: {:?}",
            tokens[2]
        );
        assert_eq!(
            tokens[3],
            Token::IntegerLiteral(123),
            "El token 3 no es un entero: {:?}",
            tokens[3]
        );
        assert_eq!(
            tokens[4],
            Token::Decl,
            "El token 4 no es un identificador: {:?}",
            tokens[4]
        );
        assert_eq!(
            tokens[5],
            Token::Identifier("y".to_string()),
            "El token 5 no es un identificador: {:?}",
            tokens[5]
        );
        assert_eq!(
            tokens[6],
            Token::Equals,
            "El token 6 no es un operador: {:?}",
            tokens[6]
        );
        assert_eq!(
            tokens[7],
            Token::IntegerLiteral(456),
            "El token 7 no es un entero: {:?}",
            tokens[7]
        );
    }

    #[test]
    fn test_while_statement() {
        let mut lexer = Lexer::new("while < á 10 do print á done".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::While,
            "El token 0 no es un while: {:?}",
            tokens[0]
        );
        assert_eq!(
            tokens[1],
            Token::Less,
            "El token 1 no es un operador <: {:?}",
            tokens[1]
        );
        assert!(matches!(tokens[2], Token::Identifier(ref s) if s == "á"));
        assert_eq!(
            tokens[3],
            Token::IntegerLiteral(10),
            "El token 3 no es un entero: {:?}",
            tokens[3]
        );

        assert_eq!(
            tokens[4],
            Token::Do,
            "El token 4 no es un do: {:?}",
            tokens[4]
        );

        assert_eq!(
            tokens[5],
            Token::Print,
            "El token 5 no es un print: {:?}",
            tokens[5]
        );

        assert!(matches!(tokens[6], Token::Identifier(ref s) if s == "á"));

        assert_eq!(
            tokens[7],
            Token::Done,
            "El token 7 no es un done: {:?}",
            tokens[7]
        );

        // sanity: ensure EOF at end
        assert!(matches!(tokens.last(), Some(Token::Eof)));
    }

    #[test]
    fn test_parentheses() {
        let mut lexer = Lexer::new("( )".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(
            tokens[0],
            Token::ParenL,
            "El token 0 no es un paréntesis izquierdo: {:?}",
            tokens[0]
        );
        assert_eq!(
            tokens[1],
            Token::ParenR,
            "El token 1 no es un paréntesis derecho: {:?}",
            tokens[1]
        );
    }

    #[test]
    fn test_parentheses_in_expression() {
        let mut lexer = Lexer::new("(+ 1 2)".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::ParenL);
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::IntegerLiteral(1));
        assert_eq!(tokens[3], Token::IntegerLiteral(2));
        assert_eq!(tokens[4], Token::ParenR);
    }

    #[test]
    fn test_nested_parentheses() {
        let mut lexer = Lexer::new("((()))".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        assert_eq!(tokens[0], Token::ParenL);
        assert_eq!(tokens[1], Token::ParenL);
        assert_eq!(tokens[2], Token::ParenL);
        assert_eq!(tokens[3], Token::ParenR);
        assert_eq!(tokens[4], Token::ParenR);
        assert_eq!(tokens[5], Token::ParenR);
    }

    #[test]
    fn test_comment_vs_parenthesis() {
        let mut lexer = Lexer::new("( (* comment *) )".to_string());
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "El lexer no debería devolver un error: {tokens:?}"
        );
        let tokens = tokens.unwrap();

        // Should only have the outer parentheses, comment should be ignored
        assert_eq!(tokens[0], Token::ParenL);
        assert_eq!(tokens[1], Token::ParenR);
        assert_eq!(tokens.len(), 3); // ParenL, ParenR, EOF
    }

    #[test]
    fn test_docs_example_smoke() {
        let src = include_str!("../docs/ejemplos.md");
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
        let mut lexer = Lexer::new(collected);
        let tokens = lexer.tokenize();
        assert!(
            tokens.is_ok(),
            "Lexer should not error on docs example: {tokens:?}"
        );
        let tokens = tokens.unwrap();
        // quick invariants
        assert!(tokens.iter().any(|t| matches!(t, Token::Decl)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Assign)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Arrow)));
        assert!(tokens.iter().any(|t| matches!(t, Token::While)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Done)));
        assert!(matches!(tokens.last(), Some(Token::Eof)));
    }
}
