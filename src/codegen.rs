use std::collections::HashMap;

use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Codegen {
    output: String,
    label_count: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            label_count: 0,
        }
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_count;
        self.label_count += 1;
        label
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
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let id = self.new_label();

                self.generate_expression(cond, vars);
                self.output.push_str("    pop rax\n");
                self.output.push_str("    cmp rax, 0\n");

                if let Some(else_stmts) = else_body {
                    self.output.push_str(&format!("    je .L_ELSE_{}\n", id));
                    for stmt in then_body {
                        self.generate_statement(&stmt, vars);
                    }
                    self.output.push_str(&format!("    jmp .L_END_{}\n", id));

                    self.output.push_str(&format!(".L_ELSE_{}:\n", id));
                    for stmt in else_stmts {
                        self.generate_statement(&stmt, vars);
                    }
                    self.output.push_str(&format!(".L_END_{}:\n", id));
                } else {
                    self.output.push_str(&format!("    je .L_END_{}\n", id));
                    for stmt in then_body {
                        self.generate_statement(&stmt, vars);
                    }
                    self.output.push_str(&format!(".L_END_{}:\n", id));
                }
            }
            Stmt::While { cond, body } => {
                let id = self.new_label();

                self.output.push_str(&format!(".L_WHILE_START_{}:\n", id));

                self.generate_expression(cond, vars);
                self.output.push_str("    pop rax\n");
                self.output.push_str("    cmp rax, 0\n");

                self.output
                    .push_str(&format!("    je .L_WHILE_END_{}\n", id));

                for stmt in body {
                    self.generate_statement(&stmt, vars);
                }

                self.output
                    .push_str(&format!("    jmp .L_WHILE_START_{}\n", id));
                self.output.push_str(&format!(".L_WHILE_END_{}:\n", id));
            }
            Stmt::Expr(expr) => {
                self.generate_expression(expr, vars);
                self.output.push_str("    pop rax\n");
            }
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
            Expr::Call { name, args } => {
                for arg in args {
                    self.generate_expression(arg, vars);
                }
                let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

                let reg_args = std::cmp::min(args.len(), 6);
                for i in (0..reg_args).rev() {
                    self.output.push_str(&format!("    pop {}\n", arg_regs[i]));
                }

                self.output.push_str(&format!("    call {}\n", name));
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
    fn test_codegen_if() {
        let input = "main() { if (1 == 1) { return 42; } return 0; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains("cmp rax, 0"));
        assert!(code.contains("je .L_END_0"));
        assert!(code.contains(".L_END_0:"));
        assert!(code.contains("mov rax, 42"));
        assert!(code.contains("mov rax, 0"));
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

    #[test]
    fn test_codegen_while() {
        let input = "main() { auto x; x = 0; while (x < 10) x = x + 1; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains(".L_WHILE_START_0:"));
        assert!(code.contains("je .L_WHILE_END_0"));
        assert!(code.contains("jmp .L_WHILE_START_0"));
        assert!(code.contains(".L_WHILE_END_0:"));
    }

    #[test]
    fn test_codegen_call() {
        let input = "main() { return foo(1, 2); }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr.program, &cr.vars);

        assert!(code.contains("mov rax, 1"));
        assert!(code.contains("mov rax, 2"));
        assert!(code.contains("pop rsi"));
        assert!(code.contains("pop rdi"));
        assert!(code.contains("call foo"));
        assert!(code.contains("push rax"));
    }
}
