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
                Some(Token::Plus)
            }
            '-' => {
                self.iter.next();
                Some(Token::Minus)
            }
            '*' => {
                self.iter.next();
                Some(Token::Star)
            }
            '/' => {
                self.iter.next();
                Some(Token::Slash)
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
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
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
