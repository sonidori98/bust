#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Auto,
    Main,
    Return,
    If,
    Else,
    While,
    Integer(i64),
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Assign,       // =
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    LParen,       // '('
    RParen,       // ')'
    LBrace,       // '{'
    RBrace,       // '}'
    Semicolon,    // ';'
    Comma,        // ','
    Identifier(String),
    Eof, // End of file
}
