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
            if let Some(t) = self.read_sign(c) {
                tokens.push(t);
                continue;
            }

            // numeric literals
            if c.is_ascii_digit() {
                tokens.push(self.read_number());
                continue
            }

            // keywords and identifiers
            if c.is_ascii_alphabetic() || c == '_' {
                tokens.push(self.read_identifier());
                continue;
            }
            panic!("Unknown token: {}, pos: {}", self.input[self.pos], self.pos);
        }

        tokens.push(Token::Eof);
        tokens
    }

    fn read_sign(&mut self, c: char) -> Option<Token> {
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
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        let num = num_str.parse::<i64>().unwrap();
        Token::Integer(num)
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == '_')
        {
            self.pos += 1;
        }
        let word: String = self.input[start..self.pos].iter().collect();
        match &word[..] {
            "main" => Token::Main,
            "return" => Token::Return,
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
}
