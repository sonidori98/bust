use crate::{codegen::Codegen, error::{emit_error, Diagnostic}, lexer::Lexer, parser::Parser};
use clap::Parser as ClapParser;
use std::io::{Read, Write};
use std::process;

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod token;

#[derive(ClapParser)]
struct Args {
    #[arg(value_name = "FILES...", required_unless_present = "string", help = "Source file (.b) followed by optional object files (.o) to link")]
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
    #[arg(
        long = "libb-path",
        value_name = "PATH",
        help = "Path to liblibb.a (default: LIBB_PATH env, or well-known paths)"
    )]
    libb_path: Option<String>,
}

fn resolve_libb(cli_path: &Option<String>) -> String {
    if let Some(path) = cli_path {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
        eprintln!("Warning: --libb-path {} not found, falling back", path);
    }

    if let Ok(path) = std::env::var("LIBB_PATH") {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    const CANDIDATES: &[&str] = &[
        "/usr/local/lib64/liblibb.a",
        "/usr/local/lib/liblibb.a",
        "/usr/lib64/liblibb.a",
        "/usr/lib/liblibb.a",
        "/usr/lib/x86_64-linux-gnu/liblibb.a",
    ];
    for path in CANDIDATES {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    let embedded = env!("LIBB_PATH");
    if std::path::Path::new(&embedded).exists() {
        return embedded.to_string();
    }

    panic!(
        "liblibb.a not found.\n\
         Install it:  make install\n\
         Or set:      export LIBB_PATH=/path/to/liblibb.a\n\
         Or pass:     --libb-path /path/to/liblibb.a"
    );
}

fn main() {
    let args = Args::parse();
    let (input, file_name) = if let Some(code_str) = args.string {
        (code_str, "<string>".to_string())
    } else {
        let file_path = &args.files[0];
        let mut file = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: failed to open input file `{}`: {}", file_path, e);
                process::exit(1);
            }
        };
        let mut code_str = String::new();
        if let Err(e) = file.read_to_string(&mut code_str) {
            eprintln!("error: failed to read input file `{}`: {}", file_path, e);
            process::exit(1);
        }
        (code_str, file_path.clone())
    };

    let tokens = match Lexer::new(&input).tokenize() {
        Ok(t) => t,
        Err(diag) => {
            emit_error(&diag, &input, &file_name);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(diag) => {
            emit_error(&diag, &input, &file_name);
            process::exit(1);
        }
    };

    let mut codegen = Codegen::new();
    let code = match codegen.generate(&program) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("error: {}", msg);
            process::exit(1);
        }
    };

    if args.assembly {
        if args.output == "a.out" {
            println!("{}", code);
        } else {
            let mut file = match std::fs::File::create(&args.output) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("error: failed to create output file `{}`: {}", args.output, e);
                    process::exit(1);
                }
            };
            if let Err(e) = file.write_all(code.as_bytes()) {
                eprintln!("error: failed to write to output file `{}`: {}", args.output, e);
                process::exit(1);
            }
        }
        return;
    }

    let libb_path = resolve_libb(&args.libb_path);

    let mut cmd = std::process::Command::new("gcc");
    cmd.arg("-x").arg("assembler").arg("-")
        .arg("-x").arg("none");

    for obj in &args.files[1..] {
        cmd.arg(obj);
    }

    cmd.arg(&libb_path)
        .arg("-o").arg(&args.output)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to start gcc: {}", e);
            process::exit(1);
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(code.as_bytes()) {
            eprintln!("error: failed to write to stdin: {}", e);
            process::exit(1);
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: failed to wait on child: {}", e);
            process::exit(1);
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        process::exit(1);
    }
}
