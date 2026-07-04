use crate::token::Token;
use crate::Diagnostic;
use std::str::Chars;

pub struct Lexer<'a> {
    source: &'a str,
    chars: Chars<'a>,
    peek: Option<char>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let peek = chars.next();
        Self {
            source: input,
            chars,
            peek,
            pos: 0,
        }
    }

    fn peek_char(&self) -> Option<&char> {
        self.peek.as_ref()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek;
        self.peek = self.chars.next();
        if let Some(c) = c {
            self.pos += c.len_utf8();
        }
        c
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, usize, usize)>, Diagnostic> {
        let mut tokens = Vec::new();

        while let Some(&c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
                continue;
            }

            if c == '/' {
                let start = self.pos;
                self.advance();
                if self.peek_char() == Some(&'*') {
                    self.advance();
                    self.skip_block_comment(start)?;
                    continue;
                }
                tokens.push((Token::Slash, start, self.pos));
                continue;
            }

            if let Some(t) = self.read_sign() {
                tokens.push(t);
                continue;
            }

            if c == '\'' {
                let start = self.pos;
                self.advance();
                let token = self.read_char_literal(start)?;
                tokens.push((token, start, self.pos));
                continue;
            }

            if c == '"' {
                let start = self.pos;
                self.advance();
                let token = self.read_string_literal(start)?;
                tokens.push((token, start, self.pos));
                continue;
            }

            if c.is_ascii_digit() {
                let start = self.pos;
                let token = self.read_number(start)?;
                tokens.push((token, start, self.pos));
                continue;
            }

            if c.is_ascii_alphabetic() || c == '_' {
                let start = self.pos;
                let token = self.read_identifier();
                tokens.push((token, start, self.pos));
                continue;
            }

            let start = self.pos;
            self.advance();
            return Err(Diagnostic::new(
                format!("unknown token `{}`", c),
                start,
                self.pos,
            ));
        }

        tokens.push((Token::Eof, self.pos, self.pos));
        Ok(tokens)
    }

    fn read_sign(&mut self) -> Option<(Token, usize, usize)> {
        let start = self.pos;
        let &c = self.peek_char()?;
        match c {
            '(' => {
                self.advance();
                Some((Token::LParen, start, self.pos))
            }
            ')' => {
                self.advance();
                Some((Token::RParen, start, self.pos))
            }
            '{' => {
                self.advance();
                Some((Token::LBrace, start, self.pos))
            }
            '}' => {
                self.advance();
                Some((Token::RBrace, start, self.pos))
            }
            ':' => {
                self.advance();
                Some((Token::Colon, start, self.pos))
            }
            ';' => {
                self.advance();
                Some((Token::Semicolon, start, self.pos))
            }
            ',' => {
                self.advance();
                Some((Token::Comma, start, self.pos))
            }
            '+' => {
                self.advance();
                if self.peek_char() == Some(&'+') {
                    self.advance();
                    Some((Token::Increment, start, self.pos))
                } else {
                    Some((Token::Plus, start, self.pos))
                }
            }
            '-' => {
                self.advance();
                if self.peek_char() == Some(&'-') {
                    self.advance();
                    Some((Token::Decrement, start, self.pos))
                } else {
                    Some((Token::Minus, start, self.pos))
                }
            }
            '*' => {
                self.advance();
                Some((Token::Star, start, self.pos))
            }
            '%' => {
                self.advance();
                Some((Token::Percent, start, self.pos))
            }
            '&' => {
                self.advance();
                Some((Token::BitAnd, start, self.pos))
            }
            '|' => {
                self.advance();
                Some((Token::BitOr, start, self.pos))
            }
            '=' => {
                self.advance();
                match self.peek_char() {
                    Some(&'=') => {
                        self.advance();
                        if self.peek_char() == Some(&'=') {
                            self.advance();
                            Some((Token::EqualAssign, start, self.pos))
                        } else {
                            Some((Token::Equal, start, self.pos))
                        }
                    }
                    Some(&'+') => {
                        self.advance();
                        Some((Token::PlusAssign, start, self.pos))
                    }
                    Some(&'-') => {
                        self.advance();
                        Some((Token::MinusAssign, start, self.pos))
                    }
                    Some(&'*') => {
                        self.advance();
                        Some((Token::MulAssign, start, self.pos))
                    }
                    Some(&'/') => {
                        self.advance();
                        Some((Token::DivAssign, start, self.pos))
                    }
                    Some(&'%') => {
                        self.advance();
                        Some((Token::ModAssign, start, self.pos))
                    }
                    Some(&'&') => {
                        self.advance();
                        Some((Token::BitAndAssign, start, self.pos))
                    }
                    Some(&'|') => {
                        self.advance();
                        Some((Token::BitOrAssign, start, self.pos))
                    }
                    Some(&'<') => {
                        self.advance();
                        if self.peek_char() == Some(&'<') {
                            self.advance();
                            Some((Token::LShiftAssign, start, self.pos))
                        } else if self.peek_char() == Some(&'=') {
                            self.advance();
                            Some((Token::LessEqualAssign, start, self.pos))
                        } else {
                            Some((Token::LessAssign, start, self.pos))
                        }
                    }
                    Some(&'>') => {
                        self.advance();
                        if self.peek_char() == Some(&'>') {
                            self.advance();
                            Some((Token::RShiftAssign, start, self.pos))
                        } else if self.peek_char() == Some(&'=') {
                            self.advance();
                            Some((Token::GreaterEqualAssign, start, self.pos))
                        } else {
                            Some((Token::GreaterAssign, start, self.pos))
                        }
                    }
                    Some(&'!') => {
                        self.advance();
                        if self.peek_char() == Some(&'=') {
                            self.advance();
                            Some((Token::NotEqualAssign, start, self.pos))
                        } else {
                            return None;
                        }
                    }
                    _ => Some((Token::Assign, start, self.pos)),
                }
            }
            '!' => {
                self.advance();
                if self.peek_char() == Some(&'=') {
                    self.advance();
                    Some((Token::NotEqual, start, self.pos))
                } else {
                    Some((Token::Not, start, self.pos))
                }
            }
            '<' => {
                self.advance();
                if self.peek_char() == Some(&'=') {
                    self.advance();
                    Some((Token::LessEqual, start, self.pos))
                } else if self.peek_char() == Some(&'<') {
                    self.advance();
                    Some((Token::LShift, start, self.pos))
                } else {
                    Some((Token::LessThan, start, self.pos))
                }
            }
            '>' => {
                self.advance();
                if self.peek_char() == Some(&'=') {
                    self.advance();
                    Some((Token::GreaterEqual, start, self.pos))
                } else if self.peek_char() == Some(&'>') {
                    self.advance();
                    Some((Token::RShift, start, self.pos))
                } else {
                    Some((Token::GreaterThan, start, self.pos))
                }
            }
            '?' => {
                self.advance();
                Some((Token::Question, start, self.pos))
            }
            '[' => {
                self.advance();
                Some((Token::LBracket, start, self.pos))
            }
            ']' => {
                self.advance();
                Some((Token::RBracket, start, self.pos))
            }
            _ => None,
        }
    }

    fn read_number(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut num_str = String::new();
        while let Some(&c) = self.peek_char() {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        let num = if num_str.len() > 1 && num_str.starts_with('0') {
            for c in num_str.chars() {
                if c < '0' || c > '7' {
                    return Err(Diagnostic::new(
                        format!("invalid octal digit `{}`", c),
                        start,
                        self.pos,
                    ));
                }
            }
            i64::from_str_radix(&num_str, 8).unwrap()
        } else {
            num_str.parse::<i64>().unwrap()
        };
        Ok(Token::Integer(num))
    }

    fn skip_block_comment(&mut self, start: usize) -> Result<(), Diagnostic> {
        while let Some(c) = self.advance() {
            if c == '*' && self.peek_char() == Some(&'/') {
                self.advance();
                return Ok(());
            }
        }
        Err(Diagnostic::new(
            "unterminated block comment",
            start,
            self.pos.min(self.source.len()),
        ))
    }

    fn read_char_literal(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut value: u64 = 0;
        let word_size = 8;

        for i in 0..word_size {
            match self.advance() {
                Some('\'') => return Ok(Token::Integer(value as i64)),
                Some('*') => {
                    value |= (self.read_escape()? as u64) << (i * 8);
                }
                Some(c) => {
                    value |= (c as u64) << (i * 8);
                }
                None => {
                    return Err(Diagnostic::new(
                        "unterminated char literal",
                        start,
                        self.pos,
                    ));
                }
            }
        }

        if self.advance() != Some('\'') {
            return Err(Diagnostic::new(
                "unterminated char literal",
                start,
                self.pos,
            ));
        }

        Ok(Token::Integer(value as i64))
    }

    fn read_escape(&mut self) -> Result<u8, Diagnostic> {
        let start = self.pos;
        match self.advance() {
            Some('0') | Some('e') => Ok(b'\0'),
            Some('(') => Ok(b'('),
            Some(')') => Ok(b')'),
            Some('*') => Ok(b'*'),
            Some('\'') => Ok(b'\''),
            Some('"') => Ok(b'"'),
            Some('t') => Ok(b'\t'),
            Some('n') => Ok(b'\n'),
            Some('r') => Ok(b'\r'),
            Some(c) => Err(Diagnostic::new(
                format!("undefined escape character `*{}`", c),
                start,
                self.pos,
            )),
            None => Err(Diagnostic::new(
                "unterminated escape sequence",
                start,
                self.pos,
            )),
        }
    }

    fn read_string_literal(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut data = Vec::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(Token::StringLiteral(data)),
                Some('*') => data.push(self.read_escape()?),
                Some(c) => data.push(c as u8),
                None => {
                    return Err(Diagnostic::new(
                        "unterminated string literal",
                        start,
                        self.pos,
                    ));
                }
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut word = String::new();
        while let Some(&c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                word.push(self.advance().unwrap());
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

    fn tokens(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|(t, _, _)| t)
            .collect()
    }

    #[test]
    fn test_lexer() {
        let input = "main() { return 42; }";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_arithmetic() {
        let input = "1 + 2 * 3 / 4 - 5";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_parentheses() {
        let input = "(1 + 2) * 3";
        assert_eq!(
            vec![Token::LParen, Token::Integer(1), Token::Plus, Token::Integer(2), Token::RParen, Token::Star, Token::Integer(3), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_increment_decrement() {
        let input = "x++ y-- ++z --w";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_not() {
        let input = "!x";
        assert_eq!(
            vec![Token::Not, Token::Identifier("x".to_string()), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_compound_assign() {
        let input = "x = 1 x == 2 x === 3 x =!= 4 x =< 5 x => 6 x =>= 7 x =<= 8";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_switch_case() {
        let input = "switch(x) { case 1: case 2: }";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_goto_label() {
        let input = "goto foo; bar:";
        assert_eq!(
            vec![
                Token::Goto,
                Token::Identifier("foo".to_string()),
                Token::Semicolon,
                Token::Identifier("bar".to_string()),
                Token::Colon,
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_block_comment() {
        let input = "1 /* comment */ 2";
        assert_eq!(vec![Token::Integer(1), Token::Integer(2), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_block_comment_empty() {
        let input = "1 /**/ 2";
        assert_eq!(vec![Token::Integer(1), Token::Integer(2), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_block_comment_multiline() {
        let input = "1 /* line1\nline2\nline3 */ 2";
        assert_eq!(vec![Token::Integer(1), Token::Integer(2), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_block_comment_special_chars() {
        let input = "1 /* == */ 2 /* ++ */ 3";
        assert_eq!(
            vec![Token::Integer(1), Token::Integer(2), Token::Integer(3), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_block_comment_unterminated() {
        let input = "1 /* unterminated";
        let err = Lexer::new(input).tokenize().unwrap_err();
        assert!(err.message.contains("unterminated block comment"));
    }

    #[test]
    fn test_lexer_char_literal_single() {
        let input = "'A'";
        assert_eq!(vec![Token::Integer(65), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_newline() {
        let input = "'*n'";
        assert_eq!(vec![Token::Integer(10), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_tab() {
        let input = "'*t'";
        assert_eq!(vec![Token::Integer(9), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_null() {
        let input = "'*0'";
        assert_eq!(vec![Token::Integer(0), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_e() {
        let input = "'*e'";
        assert_eq!(vec![Token::Integer(0), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_paren() {
        let input = "'*('";
        assert_eq!(vec![Token::Integer(40), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_star() {
        let input = "'**'";
        assert_eq!(vec![Token::Integer(42), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_escape_quote() {
        let input = "'*''";
        assert_eq!(vec![Token::Integer(39), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_multibyte() {
        let input = "'AB'";
        assert_eq!(vec![Token::Integer(16961), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_char_literal_hello() {
        let input = "'Hello'";
        assert_eq!(
            vec![Token::Integer(0x0000006F6C6C6548), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_char_literal_in_expression() {
        let input = "x = 'A';";
        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Integer(65),
                Token::Semicolon,
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_char_literal_unterminated() {
        let input = "'A";
        let err = Lexer::new(input).tokenize().unwrap_err();
        assert!(err.message.contains("unterminated char literal"));
    }

    #[test]
    fn test_lexer_char_literal_undefined_escape() {
        let input = "'*z'";
        let err = Lexer::new(input).tokenize().unwrap_err();
        assert!(err.message.contains("undefined escape character"));
    }

    #[test]
    fn test_lexer_string_literal_empty() {
        let input = "\"\"";
        assert_eq!(
            vec![Token::StringLiteral(vec![]), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_string_literal_basic() {
        let input = "\"Hello\"";
        assert_eq!(
            vec![Token::StringLiteral(b"Hello".to_vec()), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_string_literal_escape() {
        let input = "\"*n*t*r\"";
        assert_eq!(
            vec![
                Token::StringLiteral(vec![b'\n', b'\t', b'\r']),
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_string_literal_escape_null() {
        let input = "\"abc*0def\"";
        assert_eq!(
            vec![
                Token::StringLiteral(b"abc\0def".to_vec()),
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_string_literal_in_expression() {
        let input = "x = \"hi\";";
        assert_eq!(
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::StringLiteral(b"hi".to_vec()),
                Token::Semicolon,
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_string_literal_unterminated() {
        let input = "\"Hello";
        let err = Lexer::new(input).tokenize().unwrap_err();
        assert!(err.message.contains("unterminated string literal"));
    }

    #[test]
    fn test_lexer_comparison() {
        let input = "x == 1 != 2 < 3 <= 4 > 5 >= 6 if";
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
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_question() {
        let input = "1 ? 2 : 3";
        assert_eq!(
            vec![
                Token::Integer(1),
                Token::Question,
                Token::Integer(2),
                Token::Colon,
                Token::Integer(3),
                Token::Eof
            ],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_octal_zero() {
        let input = "0";
        assert_eq!(vec![Token::Integer(0), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_octal_single() {
        let input = "00";
        assert_eq!(vec![Token::Integer(0), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_octal_10() {
        let input = "010";
        assert_eq!(vec![Token::Integer(8), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_octal_777() {
        let input = "0777";
        assert_eq!(vec![Token::Integer(511), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_octal_12345() {
        let input = "012345";
        assert_eq!(vec![Token::Integer(5349), Token::Eof], tokens(input));
    }

    #[test]
    fn test_lexer_decimal_still_works() {
        let input = "10 123 999";
        assert_eq!(
            vec![Token::Integer(10), Token::Integer(123), Token::Integer(999), Token::Eof],
            tokens(input)
        );
    }

    #[test]
    fn test_lexer_octal_invalid() {
        let input = "09";
        let err = Lexer::new(input).tokenize().unwrap_err();
        assert!(err.message.contains("invalid octal digit"));
    }

    #[test]
    fn test_lexer_bracket() {
        let input = "a[0]";
        assert_eq!(
            vec![
                Token::Identifier("a".to_string()),
                Token::LBracket,
                Token::Integer(0),
                Token::RBracket,
                Token::Eof
            ],
            tokens(input)
        );
    }
}
