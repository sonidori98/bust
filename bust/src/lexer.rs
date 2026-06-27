use crate::token::Token;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            iter: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(&c) = self.iter.peek() {
            if c.is_whitespace() {
                self.iter.next();
                continue;
            }

            // block comments
            if c == '/' {
                self.iter.next();
                if self.iter.peek() == Some(&'*') {
                    self.iter.next();
                    self.skip_block_comment();
                    continue;
                }
                tokens.push(Token::Slash);
                continue;
            }

            // parentheses and delimiters
            if let Some(t) = self.read_sign() {
                tokens.push(t);
                continue;
            }

            // character literals
            if c == '\'' {
                self.iter.next();
                tokens.push(self.read_char_literal());
                continue;
            }

            // string literals
            if c == '"' {
                self.iter.next();
                tokens.push(self.read_string_literal());
                continue;
            }

            // numeric literals
            if c.is_ascii_digit() {
                tokens.push(self.read_number());
                continue;
            }

            // keywords and identifiers
            if c.is_ascii_alphabetic() || c == '_' {
                tokens.push(self.read_identifier());
                continue;
            }
            panic!("Unknown token: {}", c);
        }

        tokens.push(Token::Eof);
        tokens
    }

    fn read_sign(&mut self) -> Option<Token> {
        let &c = self.iter.peek()?;
        match c {
            '(' => {
                self.iter.next();
                Some(Token::LParen)
            }
            ')' => {
                self.iter.next();
                Some(Token::RParen)
            }
            '{' => {
                self.iter.next();
                Some(Token::LBrace)
            }
            '}' => {
                self.iter.next();
                Some(Token::RBrace)
            }
            ':' => {
                self.iter.next();
                Some(Token::Colon)
            }
            ';' => {
                self.iter.next();
                Some(Token::Semicolon)
            }
            ',' => {
                self.iter.next();
                Some(Token::Comma)
            }
            '+' => {
                self.iter.next();
                if self.iter.peek() == Some(&'+') {
                    self.iter.next();
                    Some(Token::Increment)
                } else {
                    Some(Token::Plus)
                }
            }
            '-' => {
                self.iter.next();
                if self.iter.peek() == Some(&'-') {
                    self.iter.next();
                    Some(Token::Decrement)
                } else {
                    Some(Token::Minus)
                }
            }
            '*' => {
                self.iter.next();
                Some(Token::Star)
            }
            '%' => {
                self.iter.next();
                Some(Token::Percent)
            }
            '&' => {
                self.iter.next();
                Some(Token::BitAnd)
            }
            '|' => {
                self.iter.next();
                Some(Token::BitOr)
            }
            '=' => {
                self.iter.next();
                match self.iter.peek() {
                    Some(&'=') => {
                        self.iter.next();
                        if self.iter.peek() == Some(&'=') {
                            self.iter.next();
                            Some(Token::EqualAssign)
                        } else {
                            Some(Token::Equal)
                        }
                    }
                    Some(&'+') => {
                        self.iter.next();
                        Some(Token::PlusAssign)
                    }
                    Some(&'-') => {
                        self.iter.next();
                        Some(Token::MinusAssign)
                    }
                    Some(&'*') => {
                        self.iter.next();
                        Some(Token::MulAssign)
                    }
                    Some(&'/') => {
                        self.iter.next();
                        Some(Token::DivAssign)
                    }
                    Some(&'%') => {
                        self.iter.next();
                        Some(Token::ModAssign)
                    }
                    Some(&'&') => {
                        self.iter.next();
                        Some(Token::BitAndAssign)
                    }
                    Some(&'|') => {
                        self.iter.next();
                        Some(Token::BitOrAssign)
                    }
                    Some(&'<') => {
                        self.iter.next();
                        if self.iter.peek() == Some(&'<') {
                            self.iter.next();
                            Some(Token::LShiftAssign)
                        } else if self.iter.peek() == Some(&'=') {
                            self.iter.next();
                            Some(Token::LessEqualAssign)
                        } else {
                            Some(Token::LessAssign)
                        }
                    }
                    Some(&'>') => {
                        self.iter.next();
                        if self.iter.peek() == Some(&'>') {
                            self.iter.next();
                            Some(Token::RShiftAssign)
                        } else if self.iter.peek() == Some(&'=') {
                            self.iter.next();
                            Some(Token::GreaterEqualAssign)
                        } else {
                            Some(Token::GreaterAssign)
                        }
                    }
                    Some(&'!') => {
                        self.iter.next();
                        if self.iter.peek() == Some(&'=') {
                            self.iter.next();
                            Some(Token::NotEqualAssign)
                        } else {
                            panic!("Expected '=' after '=!', but got {:?}", self.iter.peek());
                        }
                    }
                    _ => Some(Token::Assign),
                }
            }
            '!' => {
                self.iter.next();
                if self.iter.peek() == Some(&'=') {
                    self.iter.next();
                    Some(Token::NotEqual)
                } else {
                    Some(Token::Not)
                }
            }
            '<' => {
                self.iter.next();
                if self.iter.peek() == Some(&'=') {
                    self.iter.next();
                    Some(Token::LessEqual)
                } else if self.iter.peek() == Some(&'<') {
                    self.iter.next();
                    Some(Token::LShift)
                } else {
                    Some(Token::LessThan)
                }
            }
            '>' => {
                self.iter.next();
                if self.iter.peek() == Some(&'=') {
                    self.iter.next();
                    Some(Token::GreaterEqual)
                } else if self.iter.peek() == Some(&'>') {
                    self.iter.next();
                    Some(Token::RShift)
                } else {
                    Some(Token::GreaterThan)
                }
            }
            '?' => {
                self.iter.next();
                Some(Token::Question)
            }
            '[' => {
                self.iter.next();
                Some(Token::LBracket)
            }
            ']' => {
                self.iter.next();
                Some(Token::RBracket)
            }
            _ => None,
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        while let Some(&c) = self.iter.peek() {
            if c.is_ascii_digit() {
                num_str.push(self.iter.next().unwrap());
            } else {
                break;
            }
        }
        let num = if num_str.len() > 1 && num_str.starts_with('0') {
            for c in num_str.chars() {
                if c < '0' || c > '7' {
                    panic!("Invalid octal digit: {}", c);
                }
            }
            i64::from_str_radix(&num_str, 8).expect("Failed to parse octal integer")
        } else {
            num_str.parse::<i64>().expect("Failed to parse integer")
        };
        Token::Integer(num)
    }

    fn skip_block_comment(&mut self) {
        while let Some(c) = self.iter.next() {
            if c == '*' && self.iter.peek() == Some(&'/') {
                self.iter.next();
                return;
            }
        }
        panic!("Unterminated block comment");
    }

    fn read_char_literal(&mut self) -> Token {
        let mut value: u64 = 0;
        let word_size = 8;

        for i in 0..word_size {
            match self.iter.next() {
                Some('\'') => return Token::Integer(value as i64),
                Some('*') => {
                    value |= (self.read_escape() as u64) << (i * 8);
                }
                Some(c) => {
                    value |= (c as u64) << (i * 8);
                }
                None => panic!("Unterminated char literal"),
            }
        }

        if self.iter.next() != Some('\'') {
            panic!("Unterminated char literal");
        }

        Token::Integer(value as i64)
    }

    fn read_escape(&mut self) -> u8 {
        match self.iter.next() {
            Some('0') | Some('e') => b'\0',
            Some('(') => b'(',
            Some(')') => b')',
            Some('*') => b'*',
            Some('\'') => b'\'',
            Some('"') => b'"',
            Some('t') => b'\t',
            Some('n') => b'\n',
            Some('r') => b'\r',
            Some(c) => panic!("undefined escape character '*{}'", c),
            None => panic!("Unterminated escape sequence"),
        }
    }

    fn read_string_literal(&mut self) -> Token {
        let mut data = Vec::new();
        loop {
            match self.iter.next() {
                Some('"') => return Token::StringLiteral(data),
                Some('*') => data.push(self.read_escape()),
                Some(c) => data.push(c as u8),
                None => panic!("Unterminated string literal"),
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut word = String::new();
        while let Some(&c) = self.iter.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                word.push(self.iter.next().unwrap());
            } else {
                break;
            }
        }
        match &word[..] {
            "main" => Token::Main,
            "return" => Token::Return,
            "auto" => Token::Auto,
            "extrn" => Token::Extrn,
            "goto" => Token::Goto,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "switch" => Token::Switch,
            "case" => Token::Case,
            _ => Token::Identifier(word),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "main() { return 42; }";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Main,
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::Return,
                Token::Integer(42),
                Token::Semicolon,
                Token::RBrace,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_arithmetic() {
        let input = "1 + 2 * 3 / 4 - 5";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Integer(1),
                Token::Plus,
                Token::Integer(2),
                Token::Star,
                Token::Integer(3),
                Token::Slash,
                Token::Integer(4),
                Token::Minus,
                Token::Integer(5),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_parentheses() {
        let input = "(1 + 2) * 3";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::LParen,
                Token::Integer(1),
                Token::Plus,
                Token::Integer(2),
                Token::RParen,
                Token::Star,
                Token::Integer(3),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_increment_decrement() {
        let input = "x++ y-- ++z --w";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Increment,
                Token::Identifier("y".to_string()),
                Token::Decrement,
                Token::Increment,
                Token::Identifier("z".to_string()),
                Token::Decrement,
                Token::Identifier("w".to_string()),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_not() {
        let input = "!x";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![Token::Not, Token::Identifier("x".to_string()), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_compound_assign() {
        let input = "x = 1 x == 2 x === 3 x =!= 4 x =< 5 x => 6 x =>= 7 x =<= 8";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Integer(1),
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Integer(2),
                Token::Identifier("x".to_string()),
                Token::EqualAssign,
                Token::Integer(3),
                Token::Identifier("x".to_string()),
                Token::NotEqualAssign,
                Token::Integer(4),
                Token::Identifier("x".to_string()),
                Token::LessAssign,
                Token::Integer(5),
                Token::Identifier("x".to_string()),
                Token::GreaterAssign,
                Token::Integer(6),
                Token::Identifier("x".to_string()),
                Token::GreaterEqualAssign,
                Token::Integer(7),
                Token::Identifier("x".to_string()),
                Token::LessEqualAssign,
                Token::Integer(8),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_switch_case() {
        let input = "switch(x) { case 1: case 2: }";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Switch,
                Token::LParen,
                Token::Identifier("x".to_string()),
                Token::RParen,
                Token::LBrace,
                Token::Case,
                Token::Integer(1),
                Token::Colon,
                Token::Case,
                Token::Integer(2),
                Token::Colon,
                Token::RBrace,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_goto_label() {
        let input = "goto foo; bar:";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Goto,
                Token::Identifier("foo".to_string()),
                Token::Semicolon,
                Token::Identifier("bar".to_string()),
                Token::Colon,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_block_comment() {
        let input = "1 /* comment */ 2";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![Token::Integer(1), Token::Integer(2), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_block_comment_empty() {
        let input = "1 /**/ 2";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![Token::Integer(1), Token::Integer(2), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_block_comment_multiline() {
        let input = "1 /* line1\nline2\nline3 */ 2";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![Token::Integer(1), Token::Integer(2), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_block_comment_special_chars() {
        let input = "1 /* == */ 2 /* ++ */ 3";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Integer(1),
                Token::Integer(2),
                Token::Integer(3),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    #[should_panic(expected = "Unterminated block comment")]
    fn test_lexer_block_comment_unterminated() {
        let input = "1 /* unterminated";
        let mut lexer = Lexer::new(input);
        lexer.tokenize();
    }

    #[test]
    fn test_lexer_char_literal_single() {
        let input = "'A'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(65), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_newline() {
        let input = "'*n'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(10), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_tab() {
        let input = "'*t'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(9), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_null() {
        let input = "'*0'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(0), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_e() {
        let input = "'*e'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(0), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_paren() {
        let input = "'*('";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(40), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_star() {
        let input = "'**'";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(42), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_escape_quote() {
        let input = "'*''";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(39), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_multibyte() {
        let input = "'AB'";
        let mut lexer = Lexer::new(input);
        // 'A'=0x41 at byte 0, 'B'=0x42 at byte 1 → 0x4241 = 16961
        assert_eq!(vec![Token::Integer(16961), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_char_literal_hello() {
        let input = "'Hello'";
        let mut lexer = Lexer::new(input);
        // H=0x48, e=0x65, l=0x6C, l=0x6C, o=0x6F → 0x0000006F6C6C6548
        assert_eq!(
            vec![Token::Integer(0x0000006F6C6C6548), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_char_literal_in_expression() {
        let input = "x = 'A';";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Integer(65),
                Token::Semicolon,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    #[should_panic(expected = "Unterminated char literal")]
    fn test_lexer_char_literal_unterminated() {
        let input = "'A";
        let mut lexer = Lexer::new(input);
        lexer.tokenize();
    }

    #[test]
    #[should_panic(expected = "undefined escape character")]
    fn test_lexer_char_literal_undefined_escape() {
        let input = "'*z'";
        let mut lexer = Lexer::new(input);
        lexer.tokenize();
    }

    #[test]
    fn test_lexer_string_literal_empty() {
        let input = "\"\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![Token::StringLiteral(vec![]), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_string_literal_basic() {
        let input = "\"Hello\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![Token::StringLiteral(b"Hello".to_vec()), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_string_literal_escape() {
        let input = "\"*n*t*r\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::StringLiteral(vec![b'\n', b'\t', b'\r']),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_string_literal_escape_null() {
        let input = "\"abc*0def\"";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::StringLiteral(b"abc\0def".to_vec()),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_string_literal_in_expression() {
        let input = "x = \"hi\";";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::StringLiteral(b"hi".to_vec()),
                Token::Semicolon,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    #[should_panic(expected = "Unterminated string literal")]
    fn test_lexer_string_literal_unterminated() {
        let input = "\"Hello";
        let mut lexer = Lexer::new(input);
        lexer.tokenize();
    }

    #[test]
    fn test_lexer_comparison() {
        let input = "x == 1 != 2 < 3 <= 4 > 5 >= 6 if";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Integer(1),
                Token::NotEqual,
                Token::Integer(2),
                Token::LessThan,
                Token::Integer(3),
                Token::LessEqual,
                Token::Integer(4),
                Token::GreaterThan,
                Token::Integer(5),
                Token::GreaterEqual,
                Token::Integer(6),
                Token::If,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_question() {
        let input = "1 ? 2 : 3";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::Integer(1),
                Token::Question,
                Token::Integer(2),
                Token::Colon,
                Token::Integer(3),
                Token::Eof
            ],
            lexer.tokenize()
        );
    }

    #[test]
    fn test_lexer_octal_zero() {
        let input = "0";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(0), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_octal_single() {
        let input = "00";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(0), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_octal_10() {
        let input = "010";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(8), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_octal_777() {
        let input = "0777";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(511), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_octal_12345() {
        let input = "012345";
        let mut lexer = Lexer::new(input);
        assert_eq!(vec![Token::Integer(5349), Token::Eof], lexer.tokenize());
    }

    #[test]
    fn test_lexer_decimal_still_works() {
        let input = "10 123 999";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![Token::Integer(10), Token::Integer(123), Token::Integer(999), Token::Eof],
            lexer.tokenize()
        );
    }

    #[test]
    #[should_panic(expected = "Invalid octal digit")]
    fn test_lexer_octal_invalid() {
        let input = "09";
        let mut lexer = Lexer::new(input);
        lexer.tokenize();
    }

    #[test]
    fn test_lexer_bracket() {
        let input = "a[0]";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            vec![
                Token::Identifier("a".to_string()),
                Token::LBracket,
                Token::Integer(0),
                Token::RBracket,
                Token::Eof
            ],
            lexer.tokenize()
        );
    }
}
