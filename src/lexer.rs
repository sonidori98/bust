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
        let num = num_str.parse::<i64>().expect("Failed to parse integer");
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
}
