use crate::{codegen::Codegen, lexer::Lexer, parser::Parser};
use clap::Parser as ClapParser;
use std::io::{Read, Write};

mod ast;
mod codegen;
mod lexer;
mod parser;
mod token;

#[derive(ClapParser)]
struct Args {
    #[arg(value_name = "FILES", required_unless_present = "string")]
    files: Vec<String>,
    #[arg(
        short = 's',
        long = "string",
        value_name = "CODE_STRING",
        conflicts_with = "files"
    )]
    string: Option<String>,
    #[arg(short = 'o', value_name = "file", default_value = "a.out")]
    output: String,
    #[arg(short = 'S', help = "Output assembly code instead of a binary")]
    assembly: bool,
}

fn main() {
    let args = Args::parse();
    let input = if let Some(code_str) = args.string {
        code_str
    } else {
        let file_path = &args.files[0];
        let mut file = std::fs::File::open(file_path).expect("Failed to open input file");
        let mut code_str = String::new();
        file.read_to_string(&mut code_str)
            .expect("Failed to read input file");
        code_str
    };
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let mut codegen = Codegen::new();
    let program = parser.parse_program();
    let code = codegen.generate(&program);

    if args.assembly {
        if args.output == "a.out" {
            println!("{}", code);
        } else {
            let mut file = std::fs::File::create(&args.output).expect("Failed to create output file");
            file.write_all(code.as_bytes())
                .expect("Failed to write to output file");
        }
        return;
    }

    let mut child = std::process::Command::new("gcc")
        .arg("-x")
        .arg("assembler")
        .arg("-")
        .arg("-x")
        .arg("none")
        .arg(env!("LIBB_PATH"))
        .arg("-o")
        .arg(&args.output)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start gcc");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(code.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }
}
