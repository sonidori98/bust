#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}

pub fn emit_error(diag: &Diagnostic, source: &str, file: &str) {
    if diag.start >= source.len() {
        eprintln!("error: {}", diag.message);
        return;
    }

    let start = diag.start;

    let mut line = 0usize;
    let mut col = 0usize;
    for (i, c) in source.char_indices() {
        if i >= start {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += c.len_utf8();
        }
    }

    let line_num = line + 1;
    let col_display = col + 1;
    let line_text = source.lines().nth(line).unwrap_or("");

    let span_len = diag.end.saturating_sub(diag.start);
    let underline_len = if span_len == 0 {
        1
    } else {
        span_len
            .min(line_text.len().saturating_sub(col))
            .max(1)
    };

    eprintln!("error: {}", diag.message);
    eprintln!(" --> {}:{}:{}", file, line_num, col_display);
    eprintln!("  |");
    eprintln!("{:>4} | {}", line_num, line_text);
    eprintln!("  | {}{}", " ".repeat(col), "^".repeat(underline_len));
}
