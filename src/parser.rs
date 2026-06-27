use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::vec::IntoIter;

use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Parser {
    iter: Peekable<IntoIter<Token>>,
    vars: HashMap<String, i64>,
    arrays: HashSet<String>,
    global_vars: HashMap<String, String>,
    global_inits: HashMap<String, i64>,
    switch_count: usize,
    next_offset: i64,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            iter: tokens.into_iter().peekable(),
            vars: HashMap::new(),
            arrays: HashSet::new(),
            global_vars: HashMap::new(),
            global_inits: HashMap::new(),
            switch_count: 0,
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

    fn next_switch_id(&mut self) -> usize {
        self.switch_count += 1;
        self.switch_count
    }

    fn compound_op_to_binary(&self, token: Token) -> Option<Token> {
        match token {
            Token::PlusAssign => Some(Token::Plus),
            Token::MinusAssign => Some(Token::Minus),
            Token::MulAssign => Some(Token::Star),
            Token::DivAssign => Some(Token::Slash),
            Token::ModAssign => Some(Token::Percent),
            Token::BitAndAssign => Some(Token::BitAnd),
            Token::BitOrAssign => Some(Token::BitOr),
            Token::LShiftAssign => Some(Token::LShift),
            Token::RShiftAssign => Some(Token::RShift),
            Token::GreaterAssign => Some(Token::GreaterThan),
            Token::LessAssign => Some(Token::LessThan),
            Token::EqualAssign => Some(Token::Equal),
            Token::NotEqualAssign => Some(Token::NotEqual),
            Token::GreaterEqualAssign => Some(Token::GreaterEqual),
            Token::LessEqualAssign => Some(Token::LessEqual),
            _ => None,
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
            match self.peek_token().clone() {
                Token::Extrn => {
                    self.consume(Token::Extrn);
                    self.register_global();
                }
                Token::Main => {
                    self.next_token();
                    functions.push(self.parse_function_with_name("main".to_string()));
                }
                Token::Identifier(name) => {
                    self.next_token();
                    if *self.peek_token() == Token::LParen {
                        functions.push(self.parse_function_with_name(name));
                    } else {
                        self.parse_global_decl_with_name(name);
                    }
                }
                _ => panic!("Unexpected token at top level: {:?}", self.peek_token()),
            }
        }
        let globals = std::mem::take(&mut self.global_vars);
        let global_inits = std::mem::take(&mut self.global_inits);
        Program {
            functions,
            globals,
            global_inits,
        }
    }

    fn parse_global_decl_with_name(&mut self, name: String) {
        let label = format!(".{}", name);
        self.global_vars.insert(name.clone(), label);
        let expr = self.parse_expression();
        let val = match expr {
            Expr::Integer(n) => n,
            _ => panic!("Expected integer constant"),
        };
        self.global_inits.insert(name, val);
        self.consume(Token::Semicolon);
    }

    fn parse_function_with_name(&mut self, name: String) -> Function {
        self.vars.clear();
        self.next_offset = -8;

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
        let arrays = std::mem::take(&mut self.arrays);
        Function {
            name,
            params,
            body,
            locals,
            arrays,
        }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        self.consume(Token::Return);
        let expr = if *self.peek_token() == Token::Semicolon {
            Expr::Integer(0)
        } else {
            self.parse_expression()
        };
        self.consume(Token::Semicolon);
        Stmt::Return(expr)
    }

    fn parse_auto_stmt(&mut self) -> Stmt {
        self.consume(Token::Auto);
        loop {
            while *self.peek_token() == Token::Star {
                self.next_token();
            }
            if let Token::Identifier(name) = self.peek_token().clone() {
                self.next_token();
                if *self.peek_token() == Token::LBracket {
                    self.consume(Token::LBracket);
                    let size = match self.next_token() {
                        Token::Integer(n) => n,
                        _ => panic!("Expected array size"),
                    };
                    self.consume(Token::RBracket);
                    self.next_offset -= 8 * size;
                    self.vars.insert(name.clone(), self.next_offset);
                    self.arrays.insert(name);
                } else {
                    self.next_offset -= 8;
                    self.vars.insert(name, self.next_offset);
                }

                if *self.peek_token() == Token::Comma {
                    self.next_token();
                    continue;
                }
            }
            break;
        }
        self.consume(Token::Semicolon);
        Stmt::Declaration
    }

    fn parse_goto_stmt(&mut self) -> Stmt {
        self.consume(Token::Goto);
        let label = match self.next_token() {
            Token::Identifier(n) => n,
            _ => panic!("Expected label name"),
        };
        self.consume(Token::Semicolon);
        Stmt::Goto(label)
    }

    fn parse_identifier_stmt(&mut self, name: String) -> Stmt {
        self.next_token();
        match self.peek_token() {
            Token::LBracket => {
                self.consume(Token::LBracket);
                let index = self.parse_expression();
                self.consume(Token::RBracket);
                if *self.peek_token() == Token::Assign {
                    self.consume(Token::Assign);
                    let rhs = self.parse_expression();
                    self.consume(Token::Semicolon);
                    Stmt::AssignIndex(name, index, rhs)
                } else {
                    panic!("Expected '=' after array subscript");
                }
            }
            Token::Assign => {
                self.consume(Token::Assign);
                let expr = self.parse_expression();
                self.consume(Token::Semicolon);
                Stmt::Assignment(name, expr)
            }
            Token::Increment => {
                if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                    panic!("Undefined variable: {}", name);
                }
                self.next_token();
                self.consume(Token::Semicolon);
                Stmt::Expr(Expr::Postfix {
                    op: Token::Increment,
                    name,
                })
            }
            Token::Decrement => {
                if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                    panic!("Undefined variable: {}", name);
                }
                self.next_token();
                self.consume(Token::Semicolon);
                Stmt::Expr(Expr::Postfix {
                    op: Token::Decrement,
                    name,
                })
            }
            Token::LParen => {
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
            }
            Token::Colon => {
                self.consume(Token::Colon);
                Stmt::Label(name)
            }
            _ => {
                let peeked = self.peek_token().clone();
                if let Some(bin_op) = self.compound_op_to_binary(peeked) {
                    self.next_token();
                    let rhs = self.parse_expression();
                    self.consume(Token::Semicolon);
                    let desugared = Expr::Binary {
                        op: bin_op,
                        left: Box::new(Expr::Identifier(name.clone())),
                        right: Box::new(rhs),
                    };
                    Stmt::Assignment(name, desugared)
                } else {
                    panic!(
                        "Expected '=', compound assignment, '(', ':', '++', or '--' after identifier, but got {:?}",
                        self.peek_token()
                    );
                }
            }
        }
    }

    fn parse_if_stmt(&mut self) -> Stmt {
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

    fn parse_prefix_inc_stmt(&mut self) -> Stmt {
        self.next_token();
        if let Token::Identifier(name) = self.next_token() {
            if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                panic!("Undefined variable: {}", name);
            }
            self.consume(Token::Semicolon);
            Stmt::Expr(Expr::Prefix {
                op: Token::Increment,
                name,
            })
        } else {
            panic!("Expected identifier after '++'");
        }
    }

    fn parse_prefix_dec_stmt(&mut self) -> Stmt {
        self.next_token();
        if let Token::Identifier(name) = self.next_token() {
            if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                panic!("Undefined variable: {}", name);
            }
            self.consume(Token::Semicolon);
            Stmt::Expr(Expr::Prefix {
                op: Token::Decrement,
                name,
            })
        } else {
            panic!("Expected identifier after '--'");
        }
    }

    fn parse_while_stmt(&mut self) -> Stmt {
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

    fn parse_switch_stmt(&mut self) -> Stmt {
        self.consume(Token::Switch);
        self.consume(Token::LParen);
        let cond = self.parse_expression();
        self.consume(Token::RParen);

        let id = self.next_switch_id();
        let mut cases = Vec::new();
        let mut body = Vec::new();

        self.consume(Token::LBrace);
        while *self.peek_token() != Token::RBrace {
            if *self.peek_token() == Token::Case {
                self.consume(Token::Case);
                let val = match self.next_token() {
                    Token::Integer(v) => v,
                    _ => panic!("Expected integer value after case keyword"),
                };
                self.consume(Token::Colon);

                let label_name = format!("sw_{}_case_{}", id, val);
                cases.push((val, label_name.clone()));
                body.push(Stmt::Label(label_name));
            } else {
                body.push(self.parse_statement());
            }
        }
        self.consume(Token::RBrace);

        Stmt::Switch { cond, cases, body }
    }

    fn parse_statement(&mut self) -> Stmt {
        let token = self.peek_token().clone();
        match token {
            Token::Return => self.parse_return_stmt(),
            Token::Auto => self.parse_auto_stmt(),
            Token::Goto => self.parse_goto_stmt(),
            Token::Identifier(name) => self.parse_identifier_stmt(name),
            Token::If => self.parse_if_stmt(),
            Token::Increment => self.parse_prefix_inc_stmt(),
            Token::Decrement => self.parse_prefix_dec_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::Switch => self.parse_switch_stmt(),
            Token::Extrn => {
                self.consume(Token::Extrn);
                self.register_global();
                Stmt::Declaration
            }
            _ => panic!("Unsupported statement: {:?}", token),
        }
    }

    fn parse_expression(&mut self) -> Expr {
        self.parse_conditional()
    }

    fn parse_conditional(&mut self) -> Expr {
        let mut expr = self.parse_bit_or();
        if *self.peek_token() == Token::Question {
            self.consume(Token::Question);
            let then_expr = self.parse_expression();
            self.consume(Token::Colon);
            let else_expr = self.parse_expression();
            expr = Expr::Ternary {
                cond: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            }
        }
        expr
    }

    fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while *self.peek_token() == Token::BitOr {
            let op = self.next_token();
            let right = self.parse_bit_and();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while *self.peek_token() == Token::BitAnd {
            let op = self.next_token();
            let right = self.parse_equality();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
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
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
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
        let mut left = self.parse_unary();

        while matches!(
            *self.peek_token(),
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = self.next_token();
            let right = self.parse_unary();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        left
    }

    fn parse_unary_not(&mut self) -> Expr {
        self.next_token();
        let expr = self.parse_unary();
        Expr::Unary {
            op: Token::Not,
            expr: Box::new(expr),
        }
    }

    fn parse_unary_minus(&mut self) -> Expr {
        self.next_token();
        let expr = self.parse_unary();
        Expr::Unary {
            op: Token::Minus,
            expr: Box::new(expr),
        }
    }

    fn parse_unary_increment(&mut self) -> Expr {
        self.next_token();
        let expr = self.parse_unary();
        match expr {
            Expr::Identifier(name) => Expr::Prefix {
                op: Token::Increment,
                name,
            },
            _ => panic!("Invalid operand for '++'"),
        }
    }

    fn parse_unary_decrement(&mut self) -> Expr {
        self.next_token();
        let expr = self.parse_unary();
        match expr {
            Expr::Identifier(name) => Expr::Prefix {
                op: Token::Decrement,
                name,
            },
            _ => panic!("Invalid operand for '--'"),
        }
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek_token() {
            Token::Not => self.parse_unary_not(),
            Token::Minus => self.parse_unary_minus(),
            Token::Increment => self.parse_unary_increment(),
            Token::Decrement => self.parse_unary_decrement(),
            Token::Star => self.parse_unary_deref(),
            Token::BitAnd => self.parse_unary_addr(),
            _ => self.parse_primary(),
        }
    }

    fn parse_unary_deref(&mut self) -> Expr {
        self.next_token();
        Expr::Deref(Box::new(self.parse_unary()))
    }

    fn parse_unary_addr(&mut self) -> Expr {
        self.next_token();
        Expr::Addr(Box::new(self.parse_unary()))
    }

    fn parse_primary_integer(&mut self) -> Expr {
        let val = match self.next_token() {
            Token::Integer(v) => v,
            _ => unreachable!(),
        };
        Expr::Integer(val)
    }

    fn parse_primary_string_literal(&mut self) -> Expr {
        let data = match self.next_token() {
            Token::StringLiteral(d) => d,
            _ => unreachable!(),
        };
        Expr::StringLiteral(data)
    }

    fn parse_primary_identifier(&mut self, name: String) -> Expr {
        self.next_token();
        if *self.peek_token() == Token::Increment {
            if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                panic!("Undefined variable: {}", name);
            }
            self.next_token();
            Expr::Postfix {
                op: Token::Increment,
                name,
            }
        } else if *self.peek_token() == Token::Decrement {
            if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                panic!("Undefined variable: {}", name);
            }
            self.next_token();
            Expr::Postfix {
                op: Token::Decrement,
                name,
            }
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
            Expr::Call { name, args }
        } else {
            if !self.vars.contains_key(&name) && !self.global_vars.contains_key(&name) {
                panic!("Undefined variable: {}", name);
            }
            Expr::Identifier(name)
        }
    }

    fn parse_primary_paren(&mut self) -> Expr {
        self.consume(Token::LParen);
        let expr = self.parse_expression();
        self.consume(Token::RParen);
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let token = self.peek_token().clone();
        let mut expr = match token {
            Token::Integer(_) => self.parse_primary_integer(),
            Token::StringLiteral(_) => self.parse_primary_string_literal(),
            Token::Identifier(name) => self.parse_primary_identifier(name),
            Token::LParen => self.parse_primary_paren(),
            _ => panic!("Expected expression, but got {:?}", token),
        };
        loop {
            match self.peek_token() {
                Token::LBracket => {
                    self.consume(Token::LBracket);
                    let index = self.parse_expression();
                    self.consume(Token::RBracket);
                    expr = Expr::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        expr
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

        assert!(matches!(&func.body[0], Stmt::Declaration));

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
    fn test_parser_bare_return() {
        let input = "main() { return; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();
        let func = &cr.functions[0];
        assert_eq!(func.body.len(), 1);
        if let Stmt::Return(expr) = &func.body[0] {
            if let Expr::Integer(val) = expr {
                assert_eq!(*val, 0);
            } else {
                panic!("Expected Expr::Integer(0)");
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

    fn parse_one(input: &str) -> Stmt {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        program.functions[0].body[1].clone() // body[0] = auto x; body[1] = 複合代入
    }

    #[test]
    fn test_parser_plus_assign() {
        // x =+ 5  →  x = x + 5
        let stmt = parse_one("main() { auto x; x =+ 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Plus);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_minus_assign() {
        // x =- 3  →  x = x - 3
        let stmt = parse_one("main() { auto x; x =- 3; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Minus);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(3)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_mul_assign() {
        // x =* 2  →  x = x * 2
        let stmt = parse_one("main() { auto x; x =* 2; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Star);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(2)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_div_assign() {
        // x =/ 4  →  x = x / 4
        let stmt = parse_one("main() { auto x; x =/ 4; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Slash);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(4)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_mod_assign() {
        // x =% 3  →  x = x % 3
        let stmt = parse_one("main() { auto x; x =% 3; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Percent);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(3)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_bitand_assign() {
        // x =& 7  →  x = x & 7
        let stmt = parse_one("main() { auto x; x =& 7; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::BitAnd);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(7)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_bitor_assign() {
        // x =| 3  →  x = x | 3
        let stmt = parse_one("main() { auto x; x =| 3; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::BitOr);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(3)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_lshift_assign() {
        // x =<< 2  →  x = x << 2
        let stmt = parse_one("main() { auto x; x =<< 2; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::LShift);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(2)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_rshift_assign() {
        // x =>> 1  →  x = x >> 1
        let stmt = parse_one("main() { auto x; x =>> 1; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::RShift);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(1)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_less_assign() {
        let stmt = parse_one("main() { auto x; x =< 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::LessThan);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_greater_assign() {
        let stmt = parse_one("main() { auto x; x => 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::GreaterThan);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_equal_assign() {
        let stmt = parse_one("main() { auto x; x === 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Equal);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_not_equal_assign() {
        let stmt = parse_one("main() { auto x; x =!= 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::NotEqual);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_greater_equal_assign() {
        let stmt = parse_one("main() { auto x; x =>= 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::GreaterEqual);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    #[test]
    fn test_parser_less_equal_assign() {
        let stmt = parse_one("main() { auto x; x =<= 5; }");
        if let Stmt::Assignment(name, Expr::Binary { op, left, right }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::LessEqual);
            assert!(matches!(*left, Expr::Identifier(ref n) if n == "x"));
            assert!(matches!(*right, Expr::Integer(5)));
        } else {
            panic!("Expected desugared Assignment");
        }
    }

    fn body(index: usize, input: &str) -> Stmt {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        program.functions[0].body[index].clone()
    }

    #[test]
    fn test_parser_postfix_increment() {
        let stmt = body(1, "main() { auto x; x++; return x; }");
        if let Stmt::Expr(Expr::Postfix { op, name }) = stmt {
            assert_eq!(op, Token::Increment);
            assert_eq!(name, "x");
        } else {
            panic!("Expected Stmt::Expr(Expr::Postfix(Increment))");
        }
    }

    #[test]
    fn test_parser_postfix_decrement() {
        let stmt = body(1, "main() { auto x; x--; return x; }");
        if let Stmt::Expr(Expr::Postfix { op, name }) = stmt {
            assert_eq!(op, Token::Decrement);
            assert_eq!(name, "x");
        } else {
            panic!("Expected Stmt::Expr(Expr::Postfix(Decrement))");
        }
    }

    #[test]
    fn test_parser_prefix_increment() {
        let stmt = body(1, "main() { auto x; ++x; return x; }");
        if let Stmt::Expr(Expr::Prefix { op, name }) = stmt {
            assert_eq!(op, Token::Increment);
            assert_eq!(name, "x");
        } else {
            panic!("Expected Stmt::Expr(Expr::Prefix(Increment))");
        }
    }

    #[test]
    fn test_parser_prefix_decrement() {
        let stmt = body(1, "main() { auto x; --x; return x; }");
        if let Stmt::Expr(Expr::Prefix { op, name }) = stmt {
            assert_eq!(op, Token::Decrement);
            assert_eq!(name, "x");
        } else {
            panic!("Expected Stmt::Expr(Expr::Prefix(Decrement))");
        }
    }

    #[test]
    fn test_parser_postfix_in_expr() {
        let stmt = body(3, "main() { auto x; x = 1; x = x++; return x++; }");
        if let Stmt::Return(Expr::Postfix { op, name }) = stmt {
            assert_eq!(op, Token::Increment);
            assert_eq!(name, "x");
        } else {
            panic!("Expected return with Postfix(Increment)");
        }
    }

    #[test]
    fn test_parser_prefix_in_expr() {
        let stmt = body(3, "main() { auto x; x = 1; x = ++x; return ++x; }");
        if let Stmt::Return(Expr::Prefix { op, name }) = stmt {
            assert_eq!(op, Token::Increment);
            assert_eq!(name, "x");
        } else {
            panic!("Expected return with Prefix(Increment)");
        }
    }

    #[test]
    fn test_parser_incdec_global() {
        let input = "extrn x; main() { ++x; x--; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        assert!(program.globals.contains_key("x"));
        let body = &program.functions[0].body;
        assert_eq!(body.len(), 3);
        if let Stmt::Expr(Expr::Prefix { op, .. }) = &body[0] {
            assert_eq!(*op, Token::Increment);
        } else {
            panic!("Expected prefix increment");
        }
        if let Stmt::Expr(Expr::Postfix { op, .. }) = &body[1] {
            assert_eq!(*op, Token::Decrement);
        } else {
            panic!("Expected postfix decrement");
        }
    }

    #[test]
    fn test_parser_switch() {
        let input = "main() { auto x; switch(x) { case 1: return 1; case 2: return 2; } }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        let body = &program.functions[0].body;

        assert_eq!(body.len(), 2);
        assert!(matches!(&body[1], Stmt::Switch { .. }));
        if let Stmt::Switch {
            cond: _,
            cases,
            body: switch_body,
        } = &body[1]
        {
            assert_eq!(cases.len(), 2);
            assert_eq!(cases[0], (1, format!("sw_1_case_1")));
            assert_eq!(cases[1], (2, format!("sw_1_case_2")));
            assert_eq!(switch_body.len(), 4);
            assert!(matches!(&switch_body[0], Stmt::Label(n) if n == "sw_1_case_1"));
            assert!(matches!(&switch_body[2], Stmt::Label(n) if n == "sw_1_case_2"));
        }
    }

    #[test]
    fn test_parser_switch_second_id() {
        let input = "main() { switch(0) { case 0: } switch(1) { case 1: } }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        let body = &program.functions[0].body;

        assert_eq!(body.len(), 2);
        if let Stmt::Switch { cases, .. } = &body[0] {
            assert_eq!(cases[0].1, "sw_1_case_0");
        }
        if let Stmt::Switch { cases, .. } = &body[1] {
            assert_eq!(cases[0].1, "sw_2_case_1");
        }
    }

    #[test]
    fn test_parser_goto() {
        let stmt = body(1, "main() { auto x; goto end; }");
        if let Stmt::Goto(label) = stmt {
            assert_eq!(label, "end");
        } else {
            panic!("Expected Stmt::Goto");
        }
    }

    #[test]
    fn test_parser_label() {
        let stmt = body(1, "main() { auto x; loop: return x; }");
        if let Stmt::Label(label) = stmt {
            assert_eq!(label, "loop");
        } else {
            panic!("Expected Stmt::Label");
        }
    }

    #[test]
    fn test_parser_goto_label_roundtrip() {
        let input = "main() { auto x; x = 0; loop: x = x + 1; if (x < 5) goto loop; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let program = parser.parse_program();
        let body = &program.functions[0].body;

        assert_eq!(body.len(), 6);
        assert!(matches!(&body[2], Stmt::Label(n) if n == "loop"));
        assert!(matches!(&body[3], Stmt::Assignment(..)));
        if let Stmt::If {
            cond: _,
            then_body,
            else_body: _,
        } = &body[4]
        {
            assert_eq!(then_body.len(), 1);
            assert!(matches!(&then_body[0], Stmt::Goto(n) if n == "loop"));
        } else {
            panic!("Expected Stmt::If at body[4]");
        }
    }

    #[test]
    fn test_parser_unary_minus() {
        let stmt = body(1, "main() { auto x; x = -1; }");
        if let Stmt::Assignment(name, Expr::Unary { op, expr }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Minus);
            assert!(matches!(*expr, Expr::Integer(1)));
        } else {
            panic!("Expected Stmt::Assignment with Unary(Minus)");
        }
    }

    #[test]
    fn test_parser_unary_minus_chain() {
        let stmt = body(1, "main() { auto x; x = - -1; }");
        if let Stmt::Assignment(name, Expr::Unary { op, expr }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Minus);
            assert!(matches!(
                *expr,
                Expr::Unary {
                    op: Token::Minus,
                    ..
                }
            ));
        } else {
            panic!("Expected double unary minus");
        }
    }

    #[test]
    fn test_parser_unary_minus_expr() {
        let stmt = body(1, "main() { auto x, y; x = -(y + 1); }");
        if let Stmt::Assignment(name, Expr::Unary { op, expr }) = stmt {
            assert_eq!(name, "x");
            assert_eq!(op, Token::Minus);
            assert!(matches!(*expr, Expr::Binary { .. }));
        } else {
            panic!("Expected unary minus of parenthesized expr");
        }
    }

    #[test]
    fn test_parser_ternary() {
        let stmt = body(0, "main() { return 1 ? 10 : 20; }");
        if let Stmt::Return(Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        }) = stmt
        {
            assert!(matches!(*cond, Expr::Integer(1)));
            assert!(matches!(*then_expr, Expr::Integer(10)));
            assert!(matches!(*else_expr, Expr::Integer(20)));
        } else {
            panic!("Expected Stmt::Return(Expr::Ternary)");
        }
    }

    #[test]
    fn test_parser_unary_not() {
        let stmt = body(1, "main() { auto x; return !x; }");
        if let Stmt::Return(Expr::Unary { op, expr }) = stmt {
            assert_eq!(op, Token::Not);
            assert!(matches!(*expr, Expr::Identifier(ref n) if n == "x"));
        } else {
            panic!("Expected Stmt::Return(Expr::Unary(Not))");
        }
    }

    #[test]
    #[should_panic(expected = "Undefined variable")]
    fn test_parser_incdec_undefined_var_postfix() {
        let input = "main() { y++; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        parser.parse_program();
    }

    #[test]
    #[should_panic(expected = "Undefined variable")]
    fn test_parser_incdec_undefined_var_prefix() {
        let input = "main() { ++y; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        parser.parse_program();
    }

    fn parse_program(input: &str) -> crate::ast::Program {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        parser.parse_program()
    }

    #[test]
    fn test_parser_array_declaration() {
        let program = parse_program("main() { auto a[5]; }");
        assert!(program.functions[0].arrays.contains("a"));
        assert!(program.functions[0].locals.contains_key("a"));
    }

    #[test]
    fn test_parser_array_index() {
        let stmt = body(1, "main() { auto a[5]; return a[0]; }");
        if let Stmt::Return(Expr::Index { expr, index }) = stmt {
            assert!(matches!(*expr, Expr::Identifier(ref n) if n == "a"));
            assert!(matches!(*index, Expr::Integer(0)));
        } else {
            panic!("Expected Stmt::Return(Expr::Index)");
        }
    }

    #[test]
    fn test_parser_array_assign() {
        let stmt = body(1, "main() { auto a[5]; a[0] = 42; }");
        if let Stmt::AssignIndex(name, index, rhs) = stmt {
            assert_eq!(name, "a");
            assert!(matches!(index, Expr::Integer(0)));
            assert!(matches!(rhs, Expr::Integer(42)));
        } else {
            panic!("Expected Stmt::AssignIndex");
        }
    }

    #[test]
    fn test_parser_pointer_deref() {
        let stmt = body(2, "main() { auto x; auto *p; return *p; }");
        if let Stmt::Return(Expr::Deref(expr)) = stmt {
            assert!(matches!(*expr, Expr::Identifier(ref n) if n == "p"));
        } else {
            panic!("Expected Stmt::Return(Expr::Deref)");
        }
    }

    #[test]
    fn test_parser_addr_of() {
        let stmt = body(2, "main() { auto x; auto *p; p = &x; }");
        if let Stmt::Assignment(name, Expr::Addr(expr)) = stmt {
            assert_eq!(name, "p");
            assert!(matches!(*expr, Expr::Identifier(ref n) if n == "x"));
        } else {
            panic!("Expected Stmt::Assignment(Addr)");
        }
    }

    #[test]
    fn test_parser_pointer_decl() {
        let program = parse_program("main() { auto *p; auto **q; }");
        assert!(program.functions[0].locals.contains_key("p"));
        assert!(program.functions[0].locals.contains_key("q"));
    }
}
