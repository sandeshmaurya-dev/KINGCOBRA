#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // --- Keywords ---
    Let,        // let
    Fn,         // fn
    When,       // when
    Otherwise,  // otherwise
    Loop,       // loop
    Return,     // return
    Done,       // done
    While,      // while
    Import,     // import
    Class,      // class
    New,        // new

    // --- Identifiers and Data ---
    Ident(String),  // Variable names (e.g., 'score', 'player_name')
    Int(i64),       // Numbers (e.g., 10, 42)
    String(String), // Text (e.g., "Hello World")
    True,
    False,

    // --- Operators ---
    Assign,      // =
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Eq,          // ==
    NotEq,       // !=
    Lt,          // <
    Gt,          // >

    // --- Punctuation ---
    Colon,       // :
    Comma,       // ,
    Dot,         // .
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    LBrace,      // {
    RBrace,      // }

    // --- Special ---
    Illegal,     // For characters we don't recognize
    Eof,         // End of File
}
