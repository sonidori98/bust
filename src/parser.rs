use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current_token(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &Token::Eof
        }
    }

    fn consume(&mut self, expected: Token) {
        if *self.current_token() == expected {
            self.pos += 1;
        } else {
            panic!(
                "Expected {:?}, but got {:?}",
                expected,
                self.current_token()
            );
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();
        while *self.current_token() != Token::Eof {
            functions.push(self.parse_function());
        }
        Program { functions }
    }

    // "main" "(" ")" "{" ... "}"
    fn parse_function(&mut self) -> Function {
        self.consume(Token::Main);
        self.consume(Token::LParen);
        self.consume(Token::RParen);
        self.consume(Token::LBrace);

        let mut body = Vec::new();
        while *self.current_token() != Token::RBrace {
            body.push(self.parse_statement());
        }
        self.consume(Token::RBrace);
        Function {
            name: "main".to_string(),
            body,
        }
    }

    // "return" <expr> ";"
    fn parse_statement(&mut self) -> Stmt {
        match self.current_token() {
            Token::Return => {
                self.consume(Token::Return);
                let expr = self.parse_expression();
                self.consume(Token::Semicolon);
                Stmt::Return(expr)
            }
            _ => panic!("Unsupported statement: {:?}", self.current_token()),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Expr {
        let mut left = self.parse_mul_div();

        while *self.current_token() == Token::Plus || *self.current_token() == Token::Minus {
            let op = self.current_token().clone();
            self.pos += 1;
            let right = self.parse_mul_div();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_mul_div(&mut self) -> Expr {
        let mut left = self.parse_primary();

        while *self.current_token() == Token::Star || *self.current_token() == Token::Slash {
            let op = self.current_token().clone();
            self.pos += 1;
            let right = self.parse_primary();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current_token() {
            Token::Integer(val) => {
                let val = *val;
                self.pos += 1;
                Expr::Integer(val)
            }
            Token::LParen => {
                self.consume(Token::LParen);
                let expr = self.parse_expression();
                self.consume(Token::RParen);
                expr
            }
            _ => panic!("Expected expression, but got {:?}", self.current_token()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parser() {
        let input = "main() { return 42; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        assert_eq!(program.functions.len(), 1);

        let func = &program.functions[0];
        assert_eq!(func.name, "main");
        assert_eq!(func.body.len(), 1);

        let Stmt::Return(expr) = &func.body[0];
        match expr {
            Expr::Integer(val) => {
                assert_eq!(*val, 42)
            }
            _ => panic!("Expected Expr::Integer"),
        }
    }

    #[test]
    fn test_parser_arithmetic() {
        let input = "main() { return 1 + 2 * 3; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        let func = &program.functions[0];
        let Stmt::Return(expr) = &func.body[0];
        match expr {
            Expr::Binary { op, left, right } => {
                assert_eq!(*op, Token::Plus);
                match &**left {
                    Expr::Integer(val) => assert_eq!(*val, 1),
                    _ => panic!("Expected Expr::Integer(1)"),
                }
                match &**right {
                    Expr::Binary { op, left, right } => {
                        assert_eq!(*op, Token::Star);
                        match &**left {
                            Expr::Integer(val) => assert_eq!(*val, 2),
                            _ => panic!("Expected Expr::Integer(2)"),
                        }
                        match &**right {
                            Expr::Integer(val) => assert_eq!(*val, 3),
                            _ => panic!("Expected Expr::Integer(3)"),
                        }
                    }
                    _ => panic!("Expected Expr::Binary (*)"),
                }
            }
            _ => panic!("Expected Expr::Binary (+)"),
        }
    }

    #[test]
    fn test_parser_parentheses() {
        let input = "main() { return (1 + 2) * 3; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        let func = &program.functions[0];
        let Stmt::Return(expr) = &func.body[0];
        match expr {
            Expr::Binary { op, left, right } => {
                assert_eq!(*op, Token::Star);
                match &**left {
                    Expr::Binary { op, left, right } => {
                        assert_eq!(*op, Token::Plus);
                        match &**left {
                            Expr::Integer(val) => assert_eq!(*val, 1),
                            _ => panic!("Expected Expr::Integer(1)"),
                        }
                        match &**right {
                            Expr::Integer(val) => assert_eq!(*val, 2),
                            _ => panic!("Expected Expr::Integer(2)"),
                        }
                    }
                    _ => panic!("Expected Expr::Binary (+)"),
                }
                match &**right {
                    Expr::Integer(val) => assert_eq!(*val, 3),
                    _ => panic!("Expected Expr::Integer(3)"),
                }
            }
            _ => panic!("Expected Expr::Binary (*)"),
        }
    }
}
