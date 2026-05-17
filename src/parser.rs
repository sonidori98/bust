use crate::{ast::{Expr, Function, Program, Stmt}, token::Token};

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
            &Token::EOF
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
        while *self.current_token() != Token::EOF {
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
        Function { name: "main".to_string(), body }
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
        match self.current_token() {
            Token::Integer(val) => {
                let val = *val;
                self.pos += 1;
                Expr::Integer(val)
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

        match &func.body[0] {
            Stmt::Return(expr) => {
                match expr {
                    Expr::Integer(val) => {
                        assert_eq!(*val, 42)
                    },
                    _ => panic!("Expected Expr::Integer"),
                }
            }
            _ => panic!("Expected Stmt::Return"),
        }
    }
}