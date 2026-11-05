//! Hexadecimal minúsculo de digests para as cassettes sintéticas dos testes.

pub fn hex_minusculo(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut saida, byte| {
        let _ = write!(saida, "{byte:02x}");
        saida
    })
}
