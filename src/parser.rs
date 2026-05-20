use std::collections::HashMap;

use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct CompilationResult {
    pub program: Program,
    pub vars: HashMap<String, i64>,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    vars: HashMap<String, i64>,
    next_offset: i64,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            vars: HashMap::new(),
            next_offset: -8,
        }
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

    pub fn parse_program(&mut self) -> CompilationResult {
        let mut functions = Vec::new();
        while *self.current_token() != Token::Eof {
            functions.push(self.parse_function());
        }
        let program = Program { functions };
        let vars = std::mem::take(&mut self.vars);
        CompilationResult { program, vars }
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
            Token::Auto => {
                self.consume(Token::Auto);
                let mut names = Vec::new();
                loop {
                    if let Token::Identifier(name) = self.current_token().clone() {
                        self.pos += 1;
                        self.next_offset -= 8;
                        self.vars.insert(name.clone(), self.next_offset);
                        names.push(name);

                        if *self.current_token() == Token::Comma {
                            self.pos += 1;
                            continue;
                        }
                    }
                    break;
                }
                self.consume(Token::Semicolon);
                Stmt::Declaration(names)
            }
            Token::Identifier(name) => {
                if !self.vars.contains_key(name) {
                    panic!("Undefined variable: {}", name);
                }
                let name = name.clone();
                self.pos += 1;

                if *self.current_token() == Token::Assign {
                    self.consume(Token::Assign);
                    let expr = self.parse_expression();
                    self.consume(Token::Semicolon);
                    Stmt::Assignment(name, expr)
                } else {
                    panic!(
                        "Expected '=' after identifier, but got {:?}",
                        self.current_token()
                    );
                }
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
            Token::Identifier(name) => {
                if !self.vars.contains_key(name) {
                    panic!("Undefined variable: {}", name);
                }
                let name = name.clone();
                self.pos += 1;
                Expr::Identifier(name)
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
    fn test_parser_variables() {
        let input = "main() { auto x, y; x = 1; y = 2; return x + y; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.program.functions[0];
        assert_eq!(func.body.len(), 4);

        if let Stmt::Declaration(names) = &func.body[0] {
            assert_eq!(names, &vec!["x".to_string(), "y".to_string()]);
        } else {
            panic!("Expected Stmt::Declaration");
        }

        if let Stmt::Assignment(name, expr) = &func.body[1] {
            assert_eq!(name, "x");
            if let Expr::Integer(val) = expr {
                assert_eq!(*val, 1);
            } else {
                panic!("Expected Expr::Integer(1)");
            }
        } else {
            panic!("Expected Stmt::Assignment");
        }

        if let Stmt::Assignment(name, expr) = &func.body[2] {
            assert_eq!(name, "y");
            if let Expr::Integer(val) = expr {
                assert_eq!(*val, 2);
            } else {
                panic!("Expected Expr::Integer(2)");
            }
        } else {
            panic!("Expected Stmt::Assignment");
        }

        if let Stmt::Return(expr) = &func.body[3] {
            if let Expr::Binary { op, left, right } = expr {
                assert_eq!(op, &Token::Plus);
                if let Expr::Identifier(name) = &**left {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected Expr::Identifier(x)");
                }
                if let Expr::Identifier(name) = &**right {
                    assert_eq!(name, "y");
                } else {
                    panic!("Expected Expr::Identifier(y)");
                }
            } else {
                panic!("Expected Expr::Binary(+)");
            }
        } else {
            panic!("Expected Stmt::Return");
        }
    }

    #[test]
    #[should_panic(expected = "Undefined variable: y")]
    fn test_parser_undefined_variable() {
        let input = "main() { auto x; return x + y; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        parser.parse_program();
    }

    #[test]
    fn test_parser() {
        let input = "main() { return 42; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        assert_eq!(cr.program.functions.len(), 1);

        let func = &cr.program.functions[0];
        assert_eq!(func.name, "main");
        assert_eq!(func.body.len(), 1);

        if let Stmt::Return(expr) = &func.body[0] {
            match expr {
                Expr::Integer(val) => {
                    assert_eq!(*val, 42)
                }
                _ => panic!("Expected Expr::Integer"),
            }
        } else {
            panic!("Expected Stmt::Return");
        }
    }

    #[test]
    fn test_parser_arithmetic() {
        let input = "main() { return 1 + 2 * 3; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.program.functions[0];
        if let Stmt::Return(expr) = &func.body[0] {
            match expr {
                Expr::Binary { op, left, right } => {
                    assert_eq!(op, &Token::Plus);
                    match &**left {
                        Expr::Integer(val) => assert_eq!(*val, 1),
                        _ => panic!("Expected Expr::Integer(1)"),
                    }
                    match &**right {
                        Expr::Binary { op, left, right } => {
                            assert_eq!(op, &Token::Star);
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
        } else {
            panic!("Expected Stmt::Return");
        }
    }

    #[test]
    fn test_parser_parentheses() {
        let input = "main() { return (1 + 2) * 3; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.program.functions[0];
        if let Stmt::Return(expr) = &func.body[0] {
            match expr {
                Expr::Binary { op, left, right } => {
                    assert_eq!(op, &Token::Star);
                    match &**left {
                        Expr::Binary { op, left, right } => {
                            assert_eq!(op, &Token::Plus);
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
        } else {
            panic!("Expected Stmt::Return");
        }
    }
}
