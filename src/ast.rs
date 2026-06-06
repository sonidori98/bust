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
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),
    Declaration(Vec<String>),
    Assignment(String, Expr),
    If { cond: Expr, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>> },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}
