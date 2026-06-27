use std::collections::HashMap;

use crate::{
    ast::{Expr, Function, Program, Stmt},
    token::Token,
};

pub struct Codegen {
    output: String,
    current_func_name: String,
    label_count: usize,
    strings: Vec<Vec<u8>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            current_func_name: String::new(),
            label_count: 0,
            strings: Vec::new(),
        }
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_count;
        self.label_count += 1;
        label
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.push_str(".intel_syntax noprefix\n");
        self.output.push_str(".global main\n\n");

        for label in program.globals.values() {
            self.output.push_str(&format!(".comm {}, 8, 8\n", label));
        }

        for func in &program.functions {
            self.current_func_name = func.name.clone();
            self.generate_function(func, &program.globals);
        }

        if !self.strings.is_empty() {
            self.output.push_str("\n.section .rodata\n");
            for (i, data) in self.strings.iter().enumerate() {
                self.output.push_str(&format!(".string_{}:\n", i));
                self.output.push_str("    .byte ");
                let bytes: Vec<String> = data.iter().map(|b| b.to_string()).collect();
                self.output.push_str(&bytes.join(", "));
                self.output.push_str("\n");
            }
        }

        self.output.clone()
    }

    fn generate_function(&mut self, func: &Function, globals: &HashMap<String, String>) {
        self.output.push_str(&format!("{}:\n", func.name));

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        let stack_size = (func.locals.len() * 8 + 15) & !15;
        if stack_size > 0 {
            self.output
                .push_str(&format!("    sub rsp, {}\n", stack_size));
        }

        let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (i, param_name) in func.params.iter().enumerate() {
            if let Some(offset) = func.locals.get(param_name) {
                self.output
                    .push_str(&format!("    mov [rbp + {}], {}\n", offset, arg_regs[i]));
            }
        }

        for stmt in &func.body {
            self.generate_statement(stmt, &func.locals, globals);
        }

        self.output.push_str("    mov rsp, rbp\n");
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    ret\n");
    }

    fn generate_return_stmt(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(expr, locals, globals);

        self.output.push_str("    pop rax\n");
        self.output.push_str("    mov rsp, rbp\n");
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    ret\n");
    }

    fn generate_assignment_stmt(
        &mut self,
        name: &str,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(expr, locals, globals);

        self.output.push_str("    pop rax\n");

        if let Some(offset) = locals.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], rax\n", offset));
        } else if let Some(label) = globals.get(name) {
            self.output
                .push_str(&format!("    mov [rip + {}], rax\n", label));
        } else {
            panic!("Undefined variable: {}", name);
        }
    }

    fn generate_declaration_stmt(&mut self) {}

    fn generate_label_stmt(&mut self, name: &str) {
        self.output
            .push_str(&format!(".L{}_{}:\n", self.current_func_name, name));
    }

    fn generate_goto_stmt(&mut self, name: &str) {
        self.output
            .push_str(&format!("    jmp .L{}_{}\n", self.current_func_name, name));
    }

    fn generate_if_stmt(
        &mut self,
        cond: &Expr,
        then_body: &[Stmt],
        else_body: &Option<Vec<Stmt>>,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        let id = self.new_label();

        self.generate_expression(cond, locals, globals);
        self.output.push_str("    pop rax\n");
        self.output.push_str("    cmp rax, 0\n");

        if let Some(else_stmts) = else_body {
            self.output.push_str(&format!("    je .L_ELSE_{}\n", id));
                for stmt in then_body {
                    self.generate_statement(stmt, locals, globals);
                }
                self.output.push_str(&format!("    jmp .L_END_{}\n", id));

                self.output.push_str(&format!(".L_ELSE_{}:\n", id));
                for stmt in else_stmts {
                    self.generate_statement(stmt, locals, globals);
                }
                self.output.push_str(&format!(".L_END_{}:\n", id));
            } else {
                self.output.push_str(&format!("    je .L_END_{}\n", id));
                for stmt in then_body {
                    self.generate_statement(stmt, locals, globals);
                }
            self.output.push_str(&format!(".L_END_{}:\n", id));
        }
    }

    fn generate_while_stmt(
        &mut self,
        cond: &Expr,
        body: &[Stmt],
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        let id = self.new_label();

        self.output.push_str(&format!(".L_WHILE_START_{}:\n", id));

        self.generate_expression(cond, locals, globals);
        self.output.push_str("    pop rax\n");
        self.output.push_str("    cmp rax, 0\n");

        self.output
            .push_str(&format!("    je .L_WHILE_END_{}\n", id));

        for stmt in body {
            self.generate_statement(stmt, locals, globals);
        }

        self.output
            .push_str(&format!("    jmp .L_WHILE_START_{}\n", id));
        self.output.push_str(&format!(".L_WHILE_END_{}:\n", id));
    }

    fn generate_switch_stmt(
        &mut self,
        cond: &Expr,
        cases: &[(i64, String)],
        body: &[Stmt],
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(cond, locals, globals);
        self.output.push_str("    pop rax\n");

        for (val, label_name) in cases {
            self.output.push_str(&format!("    cmp rax, {}\n", val));
            self.output.push_str(&format!(
                "    je .L{}_{}\n",
                self.current_func_name, label_name
            ));
        }

        for stmt in body {
            self.generate_statement(stmt, locals, globals);
        }
    }

    fn generate_expr_stmt(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(expr, locals, globals);
        self.output.push_str("    pop rax\n");
    }

    fn generate_statement(
        &mut self,
        stmt: &Stmt,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        match stmt {
            Stmt::Return(expr) => self.generate_return_stmt(expr, locals, globals),
            Stmt::Assignment(name, expr) => self.generate_assignment_stmt(name, expr, locals, globals),
            Stmt::Declaration => self.generate_declaration_stmt(),
            Stmt::Label(name) => self.generate_label_stmt(name),
            Stmt::Goto(name) => self.generate_goto_stmt(name),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => self.generate_if_stmt(cond, then_body, else_body, locals, globals),
            Stmt::While { cond, body } => self.generate_while_stmt(cond, body, locals, globals),
            Stmt::Switch { cond, cases, body } => self.generate_switch_stmt(cond, cases, body, locals, globals),
            Stmt::Expr(expr) => self.generate_expr_stmt(expr, locals, globals),
        }
    }

    fn generate_integer_expr(&mut self, val: i64) {
        self.output.push_str(&format!("    mov rax, {}\n", val));
        self.output.push_str("    push rax\n");
    }

    fn generate_identifier_expr(
        &mut self,
        name: &str,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        if let Some(offset) = locals.get(name) {
            self.output
                .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
        } else if let Some(label) = globals.get(name) {
            self.output
                .push_str(&format!("    mov rax, [rip + {}]\n", label));
        } else {
            panic!("Undefined variable: {}", name);
        }
        self.output.push_str("    push rax\n");
    }

    fn generate_binary_op(&mut self, op: &Token) {
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
            Token::Percent => {
                self.output.push_str("    cqo\n");
                self.output.push_str("    idiv rdi\n");
                self.output.push_str("    mov rax, rdx\n");
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
            Token::BitAnd => {
                self.output.push_str("    and rax, rdi\n");
            }
            Token::BitOr => {
                self.output.push_str("    or rax, rdi\n");
            }
            Token::LShift => {
                self.output.push_str("    mov rcx, rdi\n");
                self.output.push_str("    shl rax, cl\n");
            }
            Token::RShift => {
                self.output.push_str("    mov rcx, rdi\n");
                self.output.push_str("    sar rax, cl\n");
            }
            _ => panic!("Unsupported operator: {:?}", op),
        }
    }

    fn generate_binary_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        op: &Token,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(left, locals, globals);
        self.generate_expression(right, locals, globals);

        self.output.push_str("    pop rdi\n");
        self.output.push_str("    pop rax\n");

        self.generate_binary_op(op);
        self.output.push_str("    push rax\n");
    }

    fn generate_unary_not_expr(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(expr, locals, globals);
        self.output.push_str("    pop rax\n");
        self.output.push_str("    cmp rax, 0\n");
        self.output.push_str("    sete al\n");
        self.output.push_str("    movzx rax, al\n");
        self.output.push_str("    push rax\n");
    }

    fn generate_unary_minus_expr(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(expr, locals, globals);
        self.output.push_str("    pop rax\n");
        self.output.push_str("    neg rax\n");
        self.output.push_str("    push rax\n");
    }

    fn generate_unary_expr(
        &mut self,
        op: &Token,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        match op {
            Token::Not => self.generate_unary_not_expr(expr, locals, globals),
            Token::Minus => self.generate_unary_minus_expr(expr, locals, globals),
            _ => panic!("Unsupported unary operator: {:?}", op),
        }
    }

    fn generate_prefix_expr(
        &mut self,
        op: &Token,
        name: &str,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(&Expr::Identifier(name.to_string()), locals, globals);
        self.output.push_str("    pop rax\n");
        if *op == Token::Increment {
            self.output.push_str("    add rax, 1\n");
        } else {
            self.output.push_str("    sub rax, 1\n");
        }
        self.output.push_str("    push rax\n");
        if let Some(offset) = locals.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], rax\n", offset));
        } else if let Some(label) = globals.get(name) {
            self.output
                .push_str(&format!("    mov [rip + {}], rax\n", label));
        } else {
            panic!("Undefined variable: {}", name);
        }
    }

    fn generate_postfix_expr(
        &mut self,
        op: &Token,
        name: &str,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        self.generate_expression(&Expr::Identifier(name.to_string()), locals, globals);
        self.output.push_str("    pop rax\n");
        self.output.push_str("    push rax\n");
        if *op == Token::Increment {
            self.output.push_str("    add rax, 1\n");
        } else {
            self.output.push_str("    sub rax, 1\n");
        }
        if let Some(offset) = locals.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], rax\n", offset));
        } else if let Some(label) = globals.get(name) {
            self.output
                .push_str(&format!("    mov [rip + {}], rax\n", label));
        } else {
            panic!("Undefined variable: {}", name);
        }
    }

    fn generate_call_expr(
        &mut self,
        name: &str,
        args: &[Expr],
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let reg_count = std::cmp::min(args.len(), 6);
        let stack_count = args.len().saturating_sub(6);

        for i in (6..args.len()).rev() {
            self.generate_expression(&args[i], locals, globals);
        }

        for i in 0..reg_count {
            self.generate_expression(&args[i], locals, globals);
        }

        for i in (0..reg_count).rev() {
            self.output.push_str(&format!("    pop {}\n", arg_regs[i]));
        }

        self.output.push_str(&format!("    call {}\n", name));

        if stack_count > 0 {
            self.output
                .push_str(&format!("    add rsp, {}\n", stack_count * 8));
        }

        self.output.push_str("    push rax\n");
    }

    fn generate_expression(
        &mut self,
        expr: &Expr,
        locals: &HashMap<String, i64>,
        globals: &HashMap<String, String>,
    ) {
        match expr {
            Expr::Integer(val) => self.generate_integer_expr(*val),
            Expr::Identifier(name) => self.generate_identifier_expr(name, locals, globals),
            Expr::Binary { op, left, right } => {
                self.generate_binary_expr(left, right, op, locals, globals)
            }
            Expr::Unary { op, expr } => self.generate_unary_expr(op, expr, locals, globals),
            Expr::Prefix { op, name } => self.generate_prefix_expr(op, name, locals, globals),
            Expr::Postfix { op, name } => self.generate_postfix_expr(op, name, locals, globals),
            Expr::Call { name, args } => self.generate_call_expr(name, args, locals, globals),
            Expr::StringLiteral(data) => self.generate_string_literal_expr(data),
        }
    }

    fn generate_string_literal_expr(&mut self, data: &[u8]) {
        let idx = self.strings.len();
        let mut owned = data.to_vec();
        owned.push(0); // null-terminate
        self.strings.push(owned);
        self.output
            .push_str(&format!("    lea rax, [rip + .string_{}]\n", idx));
        self.output.push_str("    push rax\n");
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
        let code = codegen.generate(&cr);

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
        let code = codegen.generate(&cr);

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
        let code = codegen.generate(&cr);

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
        let code = codegen.generate(&cr);

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
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("sub rax, rdi"));
        assert!(code.contains("idiv rdi"));
    }

    #[test]
    fn test_codegen_comparison() {
        let input = "main() { return 1 == 2; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

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
        let code = Codegen::new().generate(&cr);

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
        let code = Codegen::new().generate(&cr);

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
        let code = Codegen::new().generate(&cr);

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
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("mov rax, 1"));
        assert!(code.contains("mov rax, 2"));
        assert!(code.contains("pop rsi"));
        assert!(code.contains("pop rdi"));
        assert!(code.contains("call foo"));
        assert!(code.contains("push rax"));
    }

    #[test]
    fn test_codegen_global_shadowing() {
        let input = "extrn x; main() { auto x; x = 42; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("mov [rbp + -16], rax"));
        assert!(!code.contains("mov [rip + .x], rax"));
    }

    #[test]
    fn test_codegen_multiple_functions_global() {
        let input = "extrn g; set_g(v) { g = v; } get_g() { return g; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".comm .g, 8, 8"));
        assert!(code.contains("set_g:"));
        assert!(code.contains("mov [rip + .g], rax"));
        assert!(code.contains("get_g:"));
        assert!(code.contains("mov rax, [rip + .g]"));
    }

    #[test]
    fn test_codegen_inter_function_calls() {
        let input = "
            extrn total;
            add_to_total(n) {
                total = total + n;
                return total;
            }
            main() {
                auto res;
                res = add_to_total(10);
                res = add_to_total(20);
                return res;
            }
        ";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".comm .total, 8, 8"));
        assert!(code.contains("add_to_total:"));
        assert!(code.contains("call add_to_total"));
        assert!(code.contains("mov rax, [rip + .total]"));
        assert!(code.contains("mov [rip + .total], rax"));
    }

    #[test]
    fn test_codegen_nested_calls() {
        let input = "
            square(n) { return n * n; }
            sum_squares(a, b) {
                return square(a) + square(b);
            }
            main() {
                return sum_squares(3, 4);
            }
        ";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("square:"));
        assert!(code.contains("sum_squares:"));
        assert!(code.contains("call sum_squares"));
        assert!(code.contains("call square"));
        assert!(code.contains("imul rax, rdi"));
    }

    #[test]
    fn test_codegen_bitwise_and() {
        let input = "main() { return 12 & 10; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("and rax, rdi"));
    }

    #[test]
    fn test_codegen_bitwise_or() {
        let input = "main() { return 12 | 10; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("or rax, rdi"));
    }

    #[test]
    fn test_codegen_lshift() {
        let input = "main() { return 1 << 3; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("mov rcx, rdi"));
        assert!(code.contains("shl rax, cl"));
    }

    #[test]
    fn test_codegen_rshift() {
        let input = "main() { return 8 >> 2; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);
        assert!(code.contains("mov rcx, rdi"));
        assert!(code.contains("sar rax, cl"));
    }

    #[test]
    fn test_codegen_prefix_increment() {
        let input = "main() { auto x; x = 0; ++x; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("add rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_postfix_increment() {
        let input = "main() { auto x; x = 0; x++; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("add rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_prefix_decrement() {
        let input = "main() { auto x; x = 0; --x; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("sub rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_postfix_decrement() {
        let input = "main() { auto x; x = 0; x--; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("sub rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_prefix_inc_in_expr() {
        let input = "main() { auto x; x = 0; return ++x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("add rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_postfix_inc_in_expr() {
        let input = "main() { auto x; x = 0; return x++; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("add rax, 1"));
        assert!(code.contains("mov [rbp + -16], rax"));
    }

    #[test]
    fn test_codegen_incdec_global() {
        let input = "extrn x; main() { ++x; x--; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("add rax, 1"));
        assert!(code.contains("sub rax, 1"));
        let count_add = code.matches("add rax, 1").count();
        let count_sub = code.matches("sub rax, 1").count();
        assert!(count_add >= 1);
        assert!(count_sub >= 1);
        assert!(code.contains("mov [rip + .x], rax"));
    }

    #[test]
    fn test_codegen_switch() {
        let input = "main() { auto x; switch(x) { case 1: return 1; case 2: return 2; } }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("cmp rax, 1"));
        assert!(code.contains("cmp rax, 2"));
        assert!(code.contains("je .Lmain_sw_1_case_1"));
        assert!(code.contains("je .Lmain_sw_1_case_2"));
        assert!(code.contains(".Lmain_sw_1_case_1:"));
        assert!(code.contains(".Lmain_sw_1_case_2:"));
    }

    #[test]
    fn test_codegen_switch_fallthrough() {
        let input = "main() { auto x; switch(x) { case 0: case 1: return 1; } }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("cmp rax, 0"));
        assert!(code.contains("cmp rax, 1"));
        assert!(code.contains(".Lmain_sw_1_case_0:"));
        assert!(code.contains(".Lmain_sw_1_case_1:"));
        // case 0 falls through to case 1 (both labels before `return 1`)
        let label0_pos = code.find(".Lmain_sw_1_case_0:").unwrap();
        let label1_pos = code.find(".Lmain_sw_1_case_1:").unwrap();
        let ret_pos = code.find("mov rax, 1").unwrap();
        assert!(label0_pos < label1_pos);
        assert!(label1_pos < ret_pos);
    }

    #[test]
    fn test_codegen_goto_label() {
        let input = "main() { auto x; x = 0; loop: x = x + 1; if (x < 10) goto loop; return x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".Lmain_loop:"));
        assert!(code.contains("jmp .Lmain_loop"));
        assert!(code.contains("cmp rax, 0"));
    }

    #[test]
    fn test_codegen_goto_label_multi_function() {
        let input = "
            first() { start: goto end; end: return 0; }
            second() { top: goto top; }
        ";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".Lfirst_start:"));
        assert!(code.contains("jmp .Lfirst_end"));
        assert!(code.contains(".Lfirst_end:"));
        assert!(code.contains(".Lsecond_top:"));
        assert!(code.contains("jmp .Lsecond_top"));
    }

    #[test]
    fn test_codegen_negate_literal() {
        let input = "main() { return -42; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("neg rax"));
    }

    #[test]
    fn test_codegen_negate_var() {
        let input = "main() { auto x; x = 10; return -x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("neg rax"));
    }

    #[test]
    fn test_codegen_negate_binary() {
        let input = "main() { auto x; x = 10; return -(x + 5); }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("neg rax"));
    }

    #[test]
    fn test_codegen_unary_not() {
        let input = "main() { auto x; x = 0; return !x; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("cmp rax, 0"));
        assert!(code.contains("sete al"));
        assert!(code.contains("movzx rax, al"));
    }

    #[test]
    fn test_codegen_modulo() {
        let input = "main() { return 10 % 3; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("cqo"));
        assert!(code.contains("idiv rdi"));
        assert!(code.contains("mov rax, rdx"));
    }

    #[test]
    fn test_codegen_string_literal() {
        let input = "main() { return \"Hello\"; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains("lea rax, [rip + .string_0]"));
        assert!(code.contains(".section .rodata"));
        assert!(code.contains(".string_0:"));
        assert!(code.contains(".byte 72, 101, 108, 108, 111, 0"));
    }

    #[test]
    fn test_codegen_string_literal_escape() {
        let input = "main() { return \"*n\"; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".byte 10, 0"));
    }

    #[test]
    fn test_codegen_string_literal_empty() {
        let input = "main() { return \"\"; }";
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer.tokenize());
        let cr = parser.parse_program();
        let code = Codegen::new().generate(&cr);

        assert!(code.contains(".byte 0"));
    }
}
