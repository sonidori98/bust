use std::collections::HashMap;

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

    pub fn generate(&mut self, program: &Program, vars: &HashMap<String, i64>) -> String {
        self.output.push_str(".intel_syntax noprefix\n");
        self.output.push_str(".global main\n\n");

        for func in &program.functions {
            self.generate_function(func, vars);
        }

        self.output.clone()
    }

    fn generate_function(&mut self, func: &Function, vars: &HashMap<String, i64>) {
        self.output.push_str(&format!("{}:\n", func.name));

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        for stmt in &func.body {
            self.generate_statement(stmt, vars);
        }

        self.output.push_str("    mov rsp, rbp\n");
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    ret\n");
    }

    fn generate_statement(&mut self, stmt: &Stmt, vars: &HashMap<String, i64>) {
        match stmt {
            Stmt::Return(expr) => {
                self.generate_expression(expr, vars);

                self.output.push_str("    pop rax\n");
                self.output.push_str("    mov rsp, rbp\n");
                self.output.push_str("    pop rbp\n");
                self.output.push_str("    ret\n");
            }
            Stmt::Assignment(name, expr) => {
                self.generate_expression(expr, vars);

                self.output.push_str("    pop rax\n");

                let offset = vars.get(name).unwrap();
                self.output
                    .push_str(&format!("    mov [rbp + {}], rax\n", offset));
            }
            Stmt::Declaration(_names) => {}
        }
    }

    fn generate_expression(&mut self, expr: &Expr, vars: &HashMap<String, i64>) {
        match expr {
            Expr::Integer(val) => {
                self.output.push_str(&format!("    mov rax, {}\n", val));
                self.output.push_str("    push rax\n");
            }
            Expr::Identifier(name) => {
                let offset = vars.get(name).unwrap();
                self.output
                    .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
                self.output.push_str("    push rax\n");
            }
            Expr::Binary { op, left, right } => {
                self.generate_expression(left, vars);
                self.generate_expression(right, vars);

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
                    Token::Equal => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    sete al\n");
                        self.output.push_str("    movzx rax, al\n");
                    }
                    Token::NotEqual => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    setne al\n");
                        self.output.push_str("    movzx rax, al\n");
                    }
                    Token::LessThan => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    setl al\n");
                        self.output.push_str("    movzx rax, al\n");
                    }
                    Token::LessEqual => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    setle al\n");
                        self.output.push_str("    movzx rax, al\n");
                    }
                    Token::GreaterThan => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    setg al\n");
                        self.output.push_str("    movzx rax, al\n");
                    }
                    Token::GreaterEqual => {
                        self.output.push_str("    cmp rax, rdi\n");
                        self.output.push_str("    setge al\n");
                        self.output.push_str("    movzx rax, al\n");
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
        let cr = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&cr.program, &cr.vars);

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
"#,
            code
        )
    }

    #[test]
    fn test_codegen_arithmetic() {
        let input = "main() { return 1 + 2; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&cr.program, &cr.vars);

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
        let cr = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&cr.program, &cr.vars);

        assert!(code.contains("mov rax, 1"));
        assert!(code.contains("mov rax, 2"));
        assert!(code.contains("add rax, rdi"));
        assert!(code.contains("mov rax, 3"));
        assert!(code.contains("imul rax, rdi"));
    }

    #[test]
    fn test_codegen_variables() {
        let input = "main() { auto x; x = 42; return x; }";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let cr = parser.parse_program();

        let mut codegen = Codegen::new();
        let code = codegen.generate(&cr.program, &cr.vars);

        assert!(code.contains("mov rax, 42"));
        assert!(code.contains("mov [rbp + -16], rax"));
        assert!(code.contains("mov rax, [rbp + -16]"));
    }

    #[test]
    fn test_codegen_sub_div() {
        let input = "main() { return (10 - 2) / 4; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains("sub rax, rdi"));
        assert!(code.contains("idiv rdi"));
    }

    #[test]
    fn test_codegen_comparison() {
        let input = "main() { return 1 == 2; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains("cmp rax, rdi"));
        assert!(code.contains("sete al"));
        assert!(code.contains("movzx rax, al"));
    }

    #[test]
    fn test_codegen_complex_vars() {
        let input = "main() { auto a, b; a = 10; b = a * 2; return b + 5; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains("imul rax, rdi"));
        assert!(code.contains("add rax, rdi"));
        assert!(code.contains("[rbp + -16]"));
        assert!(code.contains("[rbp + -24]"));
    }
}
