use std::collections::HashMap;
use std::iter::Peekable;
use std::vec::IntoIter;

use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Parser {
    iter: Peekable<IntoIter<Token>>,
    vars: HashMap<String, i64>,
    global_vars: HashMap<String, String>,
    next_offset: i64,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            iter: tokens.into_iter().peekable(),
            vars: HashMap::new(),
            global_vars: HashMap::new(),
            next_offset: -8,
        }
    }

    fn peek_token(&mut self) -> &Token {
        self.iter.peek().unwrap_or(&Token::Eof)
    }

    fn next_token(&mut self) -> Token {
        self.iter.next().unwrap_or(Token::Eof)
    }

    fn consume(&mut self, expected: Token) {
        let actual = self.next_token();
        if actual != expected {
            panic!("Expected {:?}, but got {:?}", expected, actual);
        }
    }

    fn register_global(&mut self) {
        loop {
            if let Token::Identifier(name) = self.next_token() {
                let label = format!(".{}", name);
                self.global_vars.insert(name, label);

                if *self.peek_token() == Token::Comma {
                    self.consume(Token::Comma);
                } else {
                    break;
                }
            }
        }
        self.consume(Token::Semicolon);
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();
        while *self.peek_token() != Token::Eof {
            match self.peek_token() {
                Token::Extrn => {
                    self.consume(Token::Extrn);
                    self.register_global();
                }
                _ => {
                    functions.push(self.parse_function());
                }
            }
        }
        let globals = std::mem::take(&mut self.global_vars);
        Program { functions, globals }
    }

    // "func" "(" "arg" ")" "{" ... "}"
    fn parse_function(&mut self) -> Function {
        self.vars.clear();
        self.next_offset = -8;
        
        let name = match self.next_token() {
            Token::Main => "main".to_string(),
            Token::Identifier(n) => n,
            _ => panic!("Expected function name"),
        };

        self.consume(Token::LParen);
        let mut params = Vec::new();
        while *self.peek_token() != Token::RParen {
            if let Token::Identifier(p) = self.next_token() {
                self.vars.insert(p.clone(), self.next_offset);
                self.next_offset -= 8;
                params.push(p);
            }
            if *self.peek_token() == Token::Comma {
                self.consume(Token::Comma);
            }
        }
        self.consume(Token::RParen);

        self.consume(Token::LBrace);
        let mut body = Vec::new();
        while *self.peek_token() != Token::RBrace {
            body.push(self.parse_statement());
        }
        self.consume(Token::RBrace);

        let locals = std::mem::take(&mut self.vars);
        Function { name, params, body, locals }
    }

    // "return" <expr> ";"
    fn parse_statement(&mut self) -> Stmt {
        let token = self.peek_token().clone();
        match token {
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
                    if let Token::Identifier(name) = self.peek_token().clone() {
                        self.next_token();
                        self.next_offset -= 8;
                        self.vars.insert(name.clone(), self.next_offset);
                        names.push(name);

                        if *self.peek_token() == Token::Comma {
                            self.next_token();
                            continue;
                        }
                    }
                    break;
                }
                self.consume(Token::Semicolon);
                Stmt::Declaration(names)
            }
            Token::Identifier(name) => {
                self.next_token();

                if *self.peek_token() == Token::Assign {
                    self.consume(Token::Assign);
                    let expr = self.parse_expression();
                    self.consume(Token::Semicolon);
                    Stmt::Assignment(name, expr)
                } else if *self.peek_token() == Token::LParen {
                    self.consume(Token::LParen);
                    let mut args = Vec::new();
                    if *self.peek_token() != Token::RParen {
                        loop {
                            args.push(self.parse_expression());
                            if *self.peek_token() == Token::Comma {
                                self.consume(Token::Comma);
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RParen);
                    self.consume(Token::Semicolon);
                    Stmt::Expr(Expr::Call { name, args })
                } else {
                    panic!(
                        "Expected '=' or '(' after identifier, but got {:?}",
                        self.peek_token()
                    );
                }
            }
            Token::If => {
                self.consume(Token::If);
                self.consume(Token::LParen);
                let cond = self.parse_expression();
                self.consume(Token::RParen);

                let mut then_body = Vec::new();
                if *self.peek_token() == Token::LBrace {
                    self.consume(Token::LBrace);
                    while *self.peek_token() != Token::RBrace {
                        then_body.push(self.parse_statement());
                    }
                    self.consume(Token::RBrace);
                } else {
                    then_body.push(self.parse_statement());
                }
                let mut else_body = None;
                if *self.peek_token() == Token::Else {
                    self.consume(Token::Else);
                    let mut body = Vec::new();
                    if *self.peek_token() == Token::LBrace {
                        self.consume(Token::LBrace);
                        while *self.peek_token() != Token::RBrace {
                            body.push(self.parse_statement());
                        }
                        self.consume(Token::RBrace);
                    } else {
                        body.push(self.parse_statement());
                    }
                    else_body = Some(body);
                }
                Stmt::If {
                    cond,
                    then_body,
                    else_body,
                }
            }
            Token::While => {
                self.consume(Token::While);
                self.consume(Token::LParen);
                let cond = self.parse_expression();
                self.consume(Token::RParen);

                let mut body = Vec::new();
                if *self.peek_token() == Token::LBrace {
                    self.consume(Token::LBrace);
                    while *self.peek_token() != Token::RBrace {
                        body.push(self.parse_statement());
                    }
                    self.consume(Token::RBrace);
                } else {
                    body.push(self.parse_statement());
                }

                Stmt::While { cond, body }
            }
            _ => panic!("Unsupported statement: {:?}", token),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_bit_or()
    }

    fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while *self.peek_token() == Token::BitOr {
            let op = self.next_token();
            let right = self.parse_bit_and();
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
        }
        left
    }

    fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while *self.peek_token() == Token::BitAnd {
            let op = self.next_token();
            let right = self.parse_equality();
            left = Expr::Binary { op , left: Box::new(left), right: Box::new(right) }
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();

        while matches!(*self.peek_token(), Token::Equal | Token::NotEqual) {
            let op = self.next_token();
            let right = self.parse_relational();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_shift();

        while matches!(
            *self.peek_token(),
            Token::LessThan | Token::LessEqual | Token::GreaterThan | Token::GreaterEqual
        ) {
            let op = self.next_token();
            let right = self.parse_shift();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_add_sub();
        while matches!(*self.peek_token(), Token::LShift | Token::RShift) {
            let op = self.next_token();
            let right = self.parse_add_sub();
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
        }
        left
    }

    fn parse_add_sub(&mut self) -> Expr {
        let mut left = self.parse_mul_div();

        while *self.peek_token() == Token::Plus || *self.peek_token() == Token::Minus {
            let op = self.next_token();
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

        while *self.peek_token() == Token::Star || *self.peek_token() == Token::Slash {
            let op = self.next_token();
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
        let token = self.peek_token().clone();
        match token {
            Token::Integer(val) => {
                self.next_token();
                Expr::Integer(val)
            }
            Token::Identifier(name) => {
                self.next_token();
                if *self.peek_token() == Token::LParen {
                    self.consume(Token::LParen);
                    let mut args = Vec::new();

                    if *self.peek_token() != Token::RParen {
                        loop {
                            args.push(self.parse_expression());
                            if *self.peek_token() == Token::Comma {
                                self.consume(Token::Comma);
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RParen);
                    Expr::Call { name, args }
                } else {
                    if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                        panic!("Undefined variable: {}", name);
                    }
                    Expr::Identifier(name)
                }
            }
            Token::LParen => {
                self.consume(Token::LParen);
                let expr = self.parse_expression();
                self.consume(Token::RParen);
                expr
            }
            _ => panic!("Expected expression, but got {:?}", token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

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
    fn test_parser_variables() {
        let input = "main() { auto x, y; x = 1; y = 2; return x + y; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
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
    fn test_parser() {
        let input = "main() { return 42; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        assert_eq!(cr.functions.len(), 1);

        let func = &cr.functions[0];
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

        let func = &cr.functions[0];
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
    fn test_parser_if() {
        let input = "main() { if (1 == 1) { return 42; } return 0; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        assert_eq!(func.body.len(), 2);

        if let Stmt::If {
            cond,
            then_body,
            else_body,
        } = &func.body[0]
        {
            if let Expr::Binary { op, .. } = cond {
                assert_eq!(op, &Token::Equal);
            } else {
                panic!("Expected Expr::Binary for IF condition");
            }
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_none());
        } else {
            panic!("Expected Stmt::If");
        }
    }

    #[test]
    fn test_parser_comparison() {
        let input = "main() { return 1 == 2 != 3 < 4 <= 5 > 6 >= 7; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        if let Stmt::Return(expr) = &func.body[0] {
            if let Expr::Binary { op, .. } = expr {
                assert!(matches!(op, Token::Equal | Token::NotEqual));
            } else {
                panic!("Expected Expr::Binary for comparison");
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

        let func = &cr.functions[0];
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

    #[test]
    fn test_parser_if_no_brace() {
        let input = "main() { if (1 == 1) return 42; return 0; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        assert_eq!(func.body.len(), 2);

        if let Stmt::If {
            cond,
            then_body,
            else_body,
        } = &func.body[0]
        {
            if let Expr::Binary { op, .. } = cond {
                assert_eq!(op, &Token::Equal);
            } else {
                panic!("Expected Expr::Binary for IF condition");
            }
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_none());
        } else {
            panic!("Expected Stmt::If");
        }
    }

    #[test]
    fn test_parser_if_else_no_brace() {
        let input = "main() { if (1 == 1) return 42; else return 0; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        assert_eq!(func.body.len(), 1);

        if let Stmt::If {
            cond: _cond,
            then_body,
            else_body,
        } = &func.body[0]
        {
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_some());
            assert_eq!(else_body.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected Stmt::If");
        }
    }

    #[test]
    fn test_parser_while() {
        let input = "main() { while (1) { return 42; } }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        if let Stmt::While { cond: _, body } = &func.body[0] {
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected Stmt::While");
        }
    }

    #[test]
    fn test_parser_while_no_brace() {
        let input = "main() { while (1) return 42; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let func = &cr.functions[0];
        if let Stmt::While { cond: _, body } = &func.body[0] {
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected Stmt::While");
        }
    }

    #[test]
    fn test_parser_global_variables() {
        let input = "extrn a, b; extrn c; main() { return a + b + c; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        assert_eq!(program.globals.len(), 3);
        assert!(program.globals.contains_key("a"));
        assert!(program.globals.contains_key("b"));
        assert!(program.globals.contains_key("c"));

        let func = &program.functions[0];
        assert_eq!(func.name, "main");
        if let Stmt::Return(Expr::Binary { .. }) = &func.body[0] {
        } else {
            panic!("Expected return with binary expression");
        }
    }

    #[test]
    fn test_parser_global_local_mix() {
        let input = "extrn x; main() { auto x; x = 1; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();

        let func = &program.functions[0];
        assert!(func.locals.contains_key("x"));
        assert!(program.globals.contains_key("x"));
    }
}
