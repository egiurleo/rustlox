use crate::scanner::{ScanError, Scanner};

pub fn compile(source: String) {
    let mut scanner = Scanner::new(&source);

    loop {
        match scanner.scan_token() {
            Ok(token) => {
                // ...
            }
            Err(err) => {
                match err {
                    ScanError::UnexpectedChar { line } => {
                        // ...
                        break;
                    }
                }
            }
        }
    }
}
