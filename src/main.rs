use crate::{codegen::Codegen, lexer::Lexer, parser::Parser};

mod ast;
mod codegen;
mod lexer;
mod parser;
mod token;

fn main() {
    let input = "main() { return 42; }";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let mut codegen = Codegen::new();
    let program = parser.parse_program();
    let code = codegen.generate(&program);

    print!("{code}");
}
