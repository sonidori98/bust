#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Auto,
    Extrn,
    Main,
    Goto,
    Return,
    If,
    Else,
    While,
    Switch,
    Case,
    Integer(i64),
    StringLiteral(Vec<u8>),
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Percent,            // %
    BitAnd,             // &
    BitOr,              // |
    LShift,             // <<
    RShift,             // >>
    Assign,             // =
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessEqual,          // <=
    GreaterThan,        // >
    GreaterEqual,       // >=
    PlusAssign,         // =+
    MinusAssign,        // =-
    MulAssign,          // =*
    ModAssign,          // =%
    DivAssign,          // =/
    BitAndAssign,       // =&
    BitOrAssign,        // =|
    LShiftAssign,       // =<<
    RShiftAssign,       // =>>
    GreaterAssign,      // =>
    LessAssign,         // =<
    EqualAssign,        // ===
    NotEqualAssign,     // =!=
    GreaterEqualAssign, // =>=
    LessEqualAssign,    // =<=
    Not,                // !
    Increment,          // ++
    Decrement,          // --
    LParen,             // '('
    RParen,             // ')'
    LBrace,             // '{'
    RBrace,             // '}'
    Colon,              // ':'
    Semicolon,          // ';'
    Comma,              // ','
    Identifier(String),
    Eof, // End of file
}
