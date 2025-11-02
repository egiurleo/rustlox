pub fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

pub fn is_alpha(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == b'_'
}
