#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Auto,
    Extrn,
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
    BitAnd,       // &
    BitOr,        // |
    LShift,       // <<
    RShift,       // >>
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
