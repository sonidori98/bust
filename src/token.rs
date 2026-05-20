#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Main,         // "main"
    Return,       // "return"
    Integer(i64), // integer
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    LParen,       // '('
    RParen,       // ')'
    LBrace,       // '{'
    RBrace,       // '}'
    Semicolon,    // ';'
    Identifier(String),
    EOF, // End of file
}
