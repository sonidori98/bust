#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expr),
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
