#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Auto,         // "auto"
    Main,         // "main"
    Return,       // "return"
    Integer(i64), // integer
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Assign,       // =
    LParen,       // '('
    RParen,       // ')'
    LBrace,       // '{'
    RBrace,       // '}'
    Semicolon,    // ';'
    Comma,        // ','
    Identifier(String),
    Eof, // End of file
}
