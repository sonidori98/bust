use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Identifier(String),
    Binary {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Prefix {
        op: Token,
        name: String,
    },
    Postfix {
        op: Token,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),
    Declaration(Vec<String>),
    Assignment(String, Expr),
    Label(String),
    Goto(String),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Switch {
        id: usize,
        cond: Expr,
        cases: Vec<(i64, String)>,
        body: Vec<Stmt>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub locals: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub globals: std::collections::HashMap<String, String>,
}
