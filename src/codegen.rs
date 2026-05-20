use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Codegen {
    output: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.push_str(".intel_syntax noprefix\n");
        self.output.push_str(".global main\n\n");

        for func in &program.functions {
            self.generate_function(func);
        }

        self.output.clone()
    }

    fn generate_function(&mut self, func: &Function) {
        self.output.push_str(&format!("{}:\n", func.name));

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        for stmt in &func.body {
            self.generate_statement(stmt);
        }

        self.output.push_str("    mov rsp, rbp\n");
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    ret\n");
    }

    fn generate_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return(expr) => {
                self.generate_expression(expr);

                self.output.push_str("    pop rax\n");
                self.output.push_str("    mov rsp, rbp\n");
                self.output.push_str("    pop rbp\n");
                self.output.push_str("    ret\n");
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(val) => {
                self.output.push_str(&format!("    mov rax, {}\n", val));
                self.output.push_str("    push rax\n");
            }
            Expr::Binary { op, left, right } => {
                self.generate_expression(left);
                self.generate_expression(right);

                self.output.push_str("    pop rdi\n");
                self.output.push_str("    pop rax\n");

                match op {
                    Token::Plus => {
                        self.output.push_str("    add rax, rdi\n");
                    }
                    Token::Minus => {
                        self.output.push_str("    sub rax, rdi\n");
                    }
                    Token::Star => {
                        self.output.push_str("    imul rax, rdi\n");
                    }
                    Token::Slash => {
                        self.output.push_str("    cqo\n");
                        self.output.push_str("    idiv rdi\n");
                    }
                    _ => panic!("Unsupported operator: {:?}", op),
                }
                self.output.push_str("    push rax\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lexer;
    use crate::Parser;

    #[test]
    fn test_codegen() {
        let input = "main() { return 42; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&program);

        assert_eq!(
            r#".intel_syntax noprefix
.global main

main:
    push rbp
    mov rbp, rsp
    mov rax, 42
    push rax
    pop rax
    mov rsp, rbp
    pop rbp
    ret
    mov rsp, rbp
    pop rbp
    ret
"#,            code
        )
    }

    #[test]
    fn test_codegen_arithmetic() {
        let input = "main() { return 1 + 2; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&program);

        assert!(code.contains("mov rax, 1"));
        assert!(code.contains("push rax"));
        assert!(code.contains("mov rax, 2"));
        assert!(code.contains("push rax"));
        assert!(code.contains("pop rdi"));
        assert!(code.contains("pop rax"));
        assert!(code.contains("add rax, rdi"));
    }

    #[test]
    fn test_codegen_parentheses() {
        let input = "main() { return (1 + 2) * 3; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&program);

        assert!(code.contains("mov rax, 1"));
        assert!(code.contains("mov rax, 2"));
        assert!(code.contains("add rax, rdi"));
        assert!(code.contains("mov rax, 3"));
        assert!(code.contains("imul rax, rdi"));
    }
}
