use std::collections::{HashMap, HashSet};

use crate::{
    ast::{Expr, Function, GlobalArray, Program, Stmt},
    token::Token,
    Diagnostic,
};

type Spanned<T> = (T, usize, usize);

pub struct Parser {
    tokens: Vec<Spanned<Token>>,
    pos: usize,
    vars: HashMap<String, i64>,
    arrays: HashSet<String>,
    global_vars: HashMap<String, String>,
    global_inits: HashMap<String, i64>,
    global_arrays: HashMap<String, GlobalArray>,
    switch_count: usize,
    next_offset: i64,
    last_start: usize,
    last_end: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned<Token>>) -> Self {
        Self {
            tokens,
            pos: 0,
            vars: HashMap::new(),
            arrays: HashSet::new(),
            global_vars: HashMap::new(),
            global_inits: HashMap::new(),
            global_arrays: HashMap::new(),
            switch_count: 0,
            next_offset: -8,
            last_start: 0,
            last_end: 0,
        }
    }

    fn peek_token(&self) -> Token {
        self.tokens
            .get(self.pos)
            .map(|t| t.0.clone())
            .unwrap_or(Token::Eof)
    }

    fn peek_span(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|t| (t.1, t.2))
            .unwrap_or((0, 0))
    }

    fn next_spanned(&mut self) -> Spanned<Token> {
        let result = self
            .tokens
            .get(self.pos)
            .cloned()
            .unwrap_or((Token::Eof, 0, 0));
        self.pos += 1;
        self.last_start = result.1;
        self.last_end = result.2;
        result
    }

    fn next_token(&mut self) -> Token {
        self.next_spanned().0
    }

    fn consume(&mut self, expected: Token) -> Result<(), Diagnostic> {
        let (actual, start, end) = self.next_spanned();
        if actual != expected {
            return Err(Diagnostic::new(
                format!("expected `{:?}`, but found `{:?}`", expected, actual),
                start,
                end,
            ));
        }
        Ok(())
    }

    fn error(&self, msg: impl Into<String>) -> Diagnostic {
        let (start, end) = self.peek_span();
        Diagnostic::new(msg, start, end)
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

    fn register_global(&mut self) -> Result<(), Diagnostic> {
        loop {
            if let Token::Identifier(name) = self.next_token() {
                let label = format!(".{}", name);
                self.global_vars.insert(name, label);

                if self.peek_token() == Token::Comma {
                    self.consume(Token::Comma)?;
                } else {
                    break;
                }
            }
        }
        self.consume(Token::Semicolon)?;
        Ok(())
    }

    fn check_var_defined(&self, name: &str, start: usize, end: usize) -> Result<(), Diagnostic> {
        if !self.vars.contains_key(name) && !self.global_vars.contains_key(name) {
            return Err(Diagnostic::new(
                format!("undefined variable `{}`", name),
                start,
                end,
            ));
        }
        Ok(())
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut functions = Vec::new();
        while self.peek_token() != Token::Eof {
            match self.peek_token() {
                Token::Extrn => {
                    self.consume(Token::Extrn)?;
                    self.register_global()?;
                }
                Token::Main => {
                    self.next_token();
                    functions.push(self.parse_function_with_name("main".to_string())?);
                }
                Token::Identifier(name) => {
                    self.next_token();
                    if self.peek_token() == Token::LParen {
                        functions.push(self.parse_function_with_name(name)?);
                    } else {
                        self.parse_global_decl_with_name(name)?;
                    }
                }
                _ => {
                    return Err(self.error("unexpected token at top level"));
                }
            }
        }
        let globals = std::mem::take(&mut self.global_vars);
        let global_inits = std::mem::take(&mut self.global_inits);
        let global_arrays = std::mem::take(&mut self.global_arrays);
        Ok(Program {
            functions,
            globals,
            global_inits,
            global_arrays,
        })
    }

    fn parse_global_decl_with_name(&mut self, name: String) -> Result<(), Diagnostic> {
        let label = format!(".{}", name);

        if self.peek_token() == Token::LBracket {
            self.consume(Token::LBracket)?;
            let size = if self.peek_token() == Token::RBracket {
                self.consume(Token::RBracket)?;
                0
            } else {
                let n = match self.next_token() {
                    Token::Integer(n) => n,
                    _ => return Err(self.error("expected array size")),
                };
                self.consume(Token::RBracket)?;
                n
            };

            let mut init_values = Vec::new();
            if self.peek_token() != Token::Semicolon {
                loop {
                    let expr = self.parse_expression()?;
                    match expr {
                        Expr::Integer(n) => init_values.push(n),
                        _ => return Err(self.error("expected integer constant in array initializer")),
                    }
                    if self.peek_token() == Token::Comma {
                        self.consume(Token::Comma)?;
                    } else {
                        break;
                    }
                }
            }
            self.consume(Token::Semicolon)?;

            let actual_size = if size == 0 {
                init_values.len() as i64
            } else {
                size
            };

            self.global_vars.insert(name.clone(), label.clone());
            self.global_arrays.insert(
                name,
                GlobalArray {
                    label,
                    size: actual_size,
                    init_values,
                },
            );
        } else {
            self.global_vars.insert(name.clone(), label);
            if self.peek_token() != Token::Semicolon {
                let expr = self.parse_expression()?;
                let val = match expr {
                    Expr::Integer(n) => n,
                    Expr::Unary {
                        op: Token::Minus,
                        expr,
                    } if matches!(*expr, Expr::Integer(_)) => {
                        if let Expr::Integer(n) = *expr {
                            -n
                        } else {
                            unreachable!()
                        }
                    }
                    _ => return Err(self.error("expected integer constant")),
                };
                self.global_inits.insert(name, val);
            }
            self.consume(Token::Semicolon)?;
        }
        Ok(())
    }

    fn parse_function_with_name(&mut self, name: String) -> Result<Function, Diagnostic> {
        self.vars.clear();
        self.next_offset = -8;

        self.consume(Token::LParen)?;
        let mut params = Vec::new();
        while self.peek_token() != Token::RParen {
            if let Token::Identifier(p) = self.next_token() {
                self.vars.insert(p.clone(), self.next_offset);
                self.next_offset -= 8;
                params.push(p);
            }
            if self.peek_token() == Token::Comma {
                self.consume(Token::Comma)?;
            }
        }
        self.consume(Token::RParen)?;

        self.consume(Token::LBrace)?;
        let mut body = Vec::new();
        while self.peek_token() != Token::RBrace {
            body.extend(self.parse_statement()?);
        }
        self.consume(Token::RBrace)?;

        let locals = std::mem::take(&mut self.vars);
        let arrays = std::mem::take(&mut self.arrays);
        Ok(Function {
            name,
            params,
            body,
            locals,
            arrays,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.consume(Token::Return)?;
        let expr = if self.peek_token() == Token::Semicolon {
            Expr::Integer(0)
        } else {
            self.parse_expression()?
        };
        self.consume(Token::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_auto_stmt(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        self.consume(Token::Auto)?;
        let mut stmts = Vec::new();
        loop {
            while self.peek_token() == Token::Star {
                self.next_token();
            }
            if let Token::Identifier(name) = self.peek_token() {
                self.next_token();
                if self.peek_token() == Token::LBracket {
                    self.consume(Token::LBracket)?;
                    let size = match self.next_token() {
                        Token::Integer(n) => n,
                        _ => return Err(self.error("expected array size")),
                    };
                    self.consume(Token::RBracket)?;
                    self.next_offset -= 8 * size;
                    self.vars.insert(name.clone(), self.next_offset);
                    self.arrays.insert(name);
                } else {
                    self.next_offset -= 8;
                    self.vars.insert(name.clone(), self.next_offset);

                    if self.peek_token() != Token::Comma
                        && self.peek_token() != Token::Semicolon
                    {
                        let expr = self.parse_expression()?;
                        stmts.push(Stmt::Assignment(name, expr));
                    }
                }

                if self.peek_token() == Token::Comma {
                    self.next_token();
                    continue;
                }
            }
            break;
        }
        self.consume(Token::Semicolon)?;
        if stmts.is_empty() {
            Ok(vec![Stmt::Declaration])
        } else {
            Ok(stmts)
        }
    }

    fn parse_goto_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.consume(Token::Goto)?;
        let label = match self.next_token() {
            Token::Identifier(n) => n,
            _ => return Err(self.error("expected label name")),
        };
        self.consume(Token::Semicolon)?;
        Ok(Stmt::Goto(label))
    }

    fn parse_identifier_stmt(&mut self, name: String) -> Result<Stmt, Diagnostic> {
        self.next_token();
        match self.peek_token() {
            Token::LBracket => {
                self.consume(Token::LBracket)?;
                let index = self.parse_expression()?;
                self.consume(Token::RBracket)?;
                if self.peek_token() == Token::Assign {
                    self.consume(Token::Assign)?;
                    let rhs = self.parse_expression()?;
                    self.consume(Token::Semicolon)?;
                    Ok(Stmt::AssignIndex(name, index, rhs))
                } else {
                    Err(self.error("expected `=` after array subscript"))
                }
            }
            Token::Assign => {
                self.consume(Token::Assign)?;
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Assignment(name, expr))
            }
            Token::Increment => {
                self.check_var_defined(&name, self.last_start, self.last_end)?;
                self.next_token();
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::Postfix {
                    op: Token::Increment,
                    name,
                }))
            }
            Token::Decrement => {
                self.check_var_defined(&name, self.last_start, self.last_end)?;
                self.next_token();
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::Postfix {
                    op: Token::Decrement,
                    name,
                }))
            }
            Token::LParen => {
                self.consume(Token::LParen)?;
                let mut args = Vec::new();
                if self.peek_token() != Token::RParen {
                    loop {
                        args.push(self.parse_expression()?);
                        if self.peek_token() == Token::Comma {
                            self.consume(Token::Comma)?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(Token::RParen)?;
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::Call { name, args }))
            }
            Token::Colon => {
                self.consume(Token::Colon)?;
                Ok(Stmt::Label(name))
            }
            _ => {
                let peeked = self.peek_token();
                if let Some(bin_op) = self.compound_op_to_binary(peeked) {
                    self.next_token();
                    let rhs = self.parse_expression()?;
                    self.consume(Token::Semicolon)?;
                    let desugared = Expr::Binary {
                        op: bin_op,
                        left: Box::new(Expr::Identifier(name.clone())),
                        right: Box::new(rhs),
                    };
                    Ok(Stmt::Assignment(name, desugared))
                } else {
                    Err(self.error(
                        "expected `=`, compound assignment, `(`, `:`, `++`, or `--` after identifier",
                    ))
                }
            }
        }
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.consume(Token::If)?;
        self.consume(Token::LParen)?;
        let cond = self.parse_expression()?;
        self.consume(Token::RParen)?;

        let mut then_body = Vec::new();
        if self.peek_token() == Token::LBrace {
            self.consume(Token::LBrace)?;
            while self.peek_token() != Token::RBrace {
                then_body.extend(self.parse_statement()?);
            }
            self.consume(Token::RBrace)?;
        } else {
            then_body.extend(self.parse_statement()?);
        }
        let mut else_body = None;
        if self.peek_token() == Token::Else {
            self.consume(Token::Else)?;
            let mut body = Vec::new();
            if self.peek_token() == Token::LBrace {
                self.consume(Token::LBrace)?;
                while self.peek_token() != Token::RBrace {
                    body.extend(self.parse_statement()?);
                }
                self.consume(Token::RBrace)?;
            } else {
                body.extend(self.parse_statement()?);
            }
            else_body = Some(body);
        }
        Ok(Stmt::If {
            cond,
            then_body,
            else_body,
        })
    }

    fn parse_prefix_inc_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.next_token();
        let name = match self.next_token() {
            Token::Identifier(name) => name,
            _ => return Err(self.error("expected identifier after `++`")),
        };
        self.check_var_defined(&name, self.last_start, self.last_end)?;
        self.consume(Token::Semicolon)?;
        Ok(Stmt::Expr(Expr::Prefix {
            op: Token::Increment,
            name,
        }))
    }

    fn parse_prefix_dec_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.next_token();
        let name = match self.next_token() {
            Token::Identifier(name) => name,
            _ => return Err(self.error("expected identifier after `--`")),
        };
        self.check_var_defined(&name, self.last_start, self.last_end)?;
        self.consume(Token::Semicolon)?;
        Ok(Stmt::Expr(Expr::Prefix {
            op: Token::Decrement,
            name,
        }))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.consume(Token::While)?;
        self.consume(Token::LParen)?;
        let cond = self.parse_expression()?;
        self.consume(Token::RParen)?;

        let mut body = Vec::new();
        if self.peek_token() == Token::LBrace {
            self.consume(Token::LBrace)?;
            while self.peek_token() != Token::RBrace {
                body.extend(self.parse_statement()?);
            }
            self.consume(Token::RBrace)?;
        } else {
            body.extend(self.parse_statement()?);
        }

        Ok(Stmt::While { cond, body })
    }

    fn parse_switch_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        self.consume(Token::Switch)?;
        self.consume(Token::LParen)?;
        let cond = self.parse_expression()?;
        self.consume(Token::RParen)?;

        let id = self.next_switch_id();
        let mut cases = Vec::new();
        let mut body = Vec::new();

        self.consume(Token::LBrace)?;
        while self.peek_token() != Token::RBrace {
            if self.peek_token() == Token::Case {
                self.consume(Token::Case)?;
                let val = match self.next_token() {
                    Token::Integer(v) => v,
                    _ => return Err(self.error("expected integer value after `case` keyword")),
                };
                self.consume(Token::Colon)?;

                let label_name = format!("sw_{}_case_{}", id, val);
                cases.push((val, label_name.clone()));
                body.push(Stmt::Label(label_name));
            } else {
                body.extend(self.parse_statement()?);
            }
        }
        self.consume(Token::RBrace)?;

        Ok(Stmt::Switch { cond, cases, body })
    }

    fn parse_statement(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let token = self.peek_token();
        match token {
            Token::Return => Ok(vec![self.parse_return_stmt()?]),
            Token::Auto => self.parse_auto_stmt(),
            Token::Goto => Ok(vec![self.parse_goto_stmt()?]),
            Token::Identifier(name) => Ok(vec![self.parse_identifier_stmt(name)?]),
            Token::If => Ok(vec![self.parse_if_stmt()?]),
            Token::Increment => Ok(vec![self.parse_prefix_inc_stmt()?]),
            Token::Decrement => Ok(vec![self.parse_prefix_dec_stmt()?]),
            Token::While => Ok(vec![self.parse_while_stmt()?]),
            Token::Switch => Ok(vec![self.parse_switch_stmt()?]),
            Token::Extrn => {
                self.consume(Token::Extrn)?;
                self.register_global()?;
                Ok(vec![Stmt::Declaration])
            }
            _ => {
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon)?;
                Ok(vec![Stmt::Expr(expr)])
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_conditional()?;
        if self.peek_token() == Token::Assign {
            self.consume(Token::Assign)?;
            let rhs = self.parse_expression()?;
            expr = Expr::Assign {
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_conditional(&mut self) -> Result<Expr, Diagnostic> {
        let expr = self.parse_bit_or()?;
        if self.peek_token() == Token::Question {
            self.consume(Token::Question)?;
            let then_expr = self.parse_expression()?;
            self.consume(Token::Colon)?;
            let else_expr = self.parse_expression()?;
            Ok(Expr::Ternary {
                cond: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(expr)
        }
    }

    fn parse_bit_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_bit_and()?;
        while self.peek_token() == Token::BitOr {
            let op = self.next_token();
            let right = self.parse_bit_and()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_bit_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_equality()?;
        while self.peek_token() == Token::BitAnd {
            let op = self.next_token();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_relational()?;

        while matches!(self.peek_token(), Token::Equal | Token::NotEqual) {
            let op = self.next_token();
            let right = self.parse_relational()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_relational(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_shift()?;

        while matches!(
            self.peek_token(),
            Token::LessThan | Token::LessEqual | Token::GreaterThan | Token::GreaterEqual
        ) {
            let op = self.next_token();
            let right = self.parse_shift()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_add_sub()?;

        while matches!(self.peek_token(), Token::LShift | Token::RShift) {
            let op = self.next_token();
            let right = self.parse_add_sub()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_mul_div()?;

        while self.peek_token() == Token::Plus || self.peek_token() == Token::Minus {
            let op = self.next_token();
            let right = self.parse_mul_div()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, Diagnostic> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.peek_token(),
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = self.next_token();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn parse_unary_not(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        let expr = self.parse_unary()?;
        Ok(Expr::Unary {
            op: Token::Not,
            expr: Box::new(expr),
        })
    }

    fn parse_unary_minus(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        let expr = self.parse_unary()?;
        Ok(Expr::Unary {
            op: Token::Minus,
            expr: Box::new(expr),
        })
    }

    fn parse_unary_increment(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        let expr = self.parse_unary()?;
        match expr {
            Expr::Identifier(name) => Ok(Expr::Prefix {
                op: Token::Increment,
                name,
            }),
            _ => Err(self.error("invalid operand for `++`")),
        }
    }

    fn parse_unary_decrement(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        let expr = self.parse_unary()?;
        match expr {
            Expr::Identifier(name) => Ok(Expr::Prefix {
                op: Token::Decrement,
                name,
            }),
            _ => Err(self.error("invalid operand for `--`")),
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
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

    fn parse_unary_deref(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        Ok(Expr::Deref(Box::new(self.parse_unary()?)))
    }

    fn parse_unary_addr(&mut self) -> Result<Expr, Diagnostic> {
        self.next_token();
        Ok(Expr::Addr(Box::new(self.parse_unary()?)))
    }

    fn parse_primary_integer(&mut self) -> Result<Expr, Diagnostic> {
        let val = match self.next_token() {
            Token::Integer(v) => v,
            _ => unreachable!(),
        };
        Ok(Expr::Integer(val))
    }

    fn parse_primary_string_literal(&mut self) -> Result<Expr, Diagnostic> {
        let data = match self.next_token() {
            Token::StringLiteral(d) => d,
            _ => unreachable!(),
        };
        Ok(Expr::StringLiteral(data))
    }

    fn parse_primary_identifier(&mut self, name: String) -> Result<Expr, Diagnostic> {
        self.next_token();
        if self.peek_token() == Token::Increment {
            self.check_var_defined(&name, self.last_start, self.last_end)?;
            self.next_token();
            Ok(Expr::Postfix {
                op: Token::Increment,
                name,
            })
        } else if self.peek_token() == Token::Decrement {
            self.check_var_defined(&name, self.last_start, self.last_end)?;
            self.next_token();
            Ok(Expr::Postfix {
                op: Token::Decrement,
                name,
            })
        } else if self.peek_token() == Token::LParen {
            self.consume(Token::LParen)?;
            let mut args = Vec::new();

            if self.peek_token() != Token::RParen {
                loop {
                    args.push(self.parse_expression()?);
                    if self.peek_token() == Token::Comma {
                        self.consume(Token::Comma)?;
                    } else {
                        break;
                    }
                }
            }
            self.consume(Token::RParen)?;
            Ok(Expr::Call { name, args })
        } else {
            self.check_var_defined(&name, self.last_start, self.last_end)?;
            Ok(Expr::Identifier(name))
        }
    }

    fn parse_primary_paren(&mut self) -> Result<Expr, Diagnostic> {
        self.consume(Token::LParen)?;
        let expr = self.parse_expression()?;
        self.consume(Token::RParen)?;
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.peek_token();
        let (start, end) = self.peek_span();
        let mut expr = match token {
            Token::Integer(_) => self.parse_primary_integer()?,
            Token::StringLiteral(_) => self.parse_primary_string_literal()?,
            Token::Identifier(name) => self.parse_primary_identifier(name)?,
            Token::LParen => self.parse_primary_paren()?,
            _ => {
                return Err(Diagnostic::new(
                    format!("expected expression, but found `{:?}`", self.peek_token()),
                    start,
                    end,
                ));
            }
        };
        loop {
            match self.peek_token() {
                Token::LBracket => {
                    self.consume(Token::LBracket)?;
                    let index = self.parse_expression()?;
                    self.consume(Token::RBracket)?;
                    expr = Expr::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn tokenize(input: &str) -> Vec<(Token, usize, usize)> {
        Lexer::new(input).tokenize().unwrap()
    }

    #[test]
    fn test_parser_undefined_variable() {
        let input = "main() { auto x; return x + y; }";
        let mut parser = Parser::new(tokenize(input));
        let err = parser.parse_program().unwrap_err();
        assert!(err.message.contains("undefined variable"));
    }

    #[test]
    fn test_parser_variables() {
        let input = "main() { auto x, y; x = 1; y = 2; return x + y; }";
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();
        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        assert_eq!(program.functions.len(), 1);

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
        if let Stmt::While { cond: _, body } = &func.body[0] {
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected Stmt::While");
        }
    }

    #[test]
    fn test_parser_while_no_brace() {
        let input = "main() { while (1) return 42; }";
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
        if let Stmt::While { cond: _, body } = &func.body[0] {
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected Stmt::While");
        }
    }

    #[test]
    fn test_parser_global_variables() {
        let input = "extrn a, b; extrn c; main() { return a + b + c; }";
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();

        let func = &program.functions[0];
        assert!(func.locals.contains_key("x"));
        assert!(program.globals.contains_key("x"));
    }

    fn parse_one(input: &str) -> Stmt {
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();
        program.functions[0].body[1].clone()
    }

    #[test]
    fn test_parser_plus_assign() {
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();
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
        let mut parser = Parser::new(tokenize(input));
        let program = parser.parse_program().unwrap();
        let body = &program.functions[0].body;

        assert_eq!(body.len(), 2);
        assert!(matches!(&body[1], Stmt::Switch { .. }));
    }
}
