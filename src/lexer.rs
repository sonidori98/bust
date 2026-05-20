use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.pos < self.input.len() {
            let c = self.input[self.pos];

            if c.is_whitespace() {
                self.pos += 1;
                continue;
            }

            // parentheses and delimiters
            let token = match c {
                '(' => Some(Token::LParen),
                ')' => Some(Token::RParen),
                '{' => Some(Token::LBrace),
                '}' => Some(Token::RBrace),
                ';' => Some(Token::Semicolon),
                '+' => Some(Token::Plus),
                '-' => Some(Token::Minus),
                '*' => Some(Token::Star),
                '/' => Some(Token::Slash),
                _ => None,
            };
            if let Some(t) = token {
                tokens.push(t);
                self.pos += 1;
                continue;
            }

            // numeric literals
            if c.is_ascii_digit() {
                let start = self.pos;
                while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                let num_str: String = self.input[start..self.pos].iter().collect();
                let num = num_str.parse::<i64>().unwrap();
                tokens.push(Token::Integer(num));
                continue;
            }

            // keywords and identifiers
            if c.is_ascii_alphabetic() || c == '_' {
                let start = self.pos;
                while self.pos < self.input.len()
                    && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == '_')
                {
                    self.pos += 1;
                }
                let word: String = self.input[start..self.pos].iter().collect();
                let token = match &word[..] {
                    "main" => Token::Main,
                    "return" => Token::Return,
                    _ => Token::Identifier(word),
                };
                tokens.push(token);
                continue;
            }
            panic!("Unknown token: {}, pos: {}", self.input[self.pos], self.pos);
        }

        tokens.push(Token::EOF);
        tokens
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
                Token::EOF
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
                Token::EOF
            ],
            lexer.tokenize()
        );
    }
}
