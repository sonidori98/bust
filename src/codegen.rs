use crate::ast::{Expr, Function, Program, Stmt};

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
}
