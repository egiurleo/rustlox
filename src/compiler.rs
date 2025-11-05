use crate::scanner::{ScanError, Scanner, TokenType};
use std::io::Write;

pub fn compile<W: Write>(source: String, writer: &mut W) {
    let mut scanner = Scanner::new(&source);
    let mut line: usize = usize::MAX;

    loop {
        match scanner.scan_token() {
            Ok(token) => {
                if token.line != line {
                    write!(writer, "{:4} ", token.line).unwrap();
                    line = token.line;
                } else {
                    write!(writer, "   | ").unwrap();
                }

                writeln!(
                    writer,
                    "{:2} '{}'",
                    token.token_type as i32,
                    &source[token.start..token.start + token.length]
                )
                .unwrap();

                if token.token_type == TokenType::Eof {
                    break;
                }
            }
            Err(err) => match err {
                ScanError::UnexpectedChar { line } => {
                    writeln!(writer, "Unexpected char on line: {}", line).unwrap();
                }
                ScanError::UnterminatedString { line } => {
                    writeln!(writer, "Unterminated string on line: {}", line).unwrap();
                }
            },
        }
    }
}
