#[derive(Default)]
pub struct Scanner {
    line: usize,
    start: usize,
    current: usize,
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            line: 1,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_scanner_test() {
        let source = "Hello, world!".to_string();
        let scanner = Scanner::new(source);

        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.start, 0);
        assert_eq!(scanner.current, 0);
        assert_eq!(scanner.source, "Hello, world!");
    }
}
