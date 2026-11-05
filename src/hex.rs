//! Hexadecimal minúsculo de digests (`sha1`/`sha2` 0.11 não implementam `LowerHex`).

const DIGITOS: &[u8; 16] = b"0123456789abcdef";

/// Dois dígitos minúsculos por byte, na ordem dos bytes.
pub(crate) fn hex_minusculo(bytes: &[u8]) -> String {
    let mut saida = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        saida.push(char::from(DIGITOS[usize::from(byte >> 4)]));
        saida.push(char::from(DIGITOS[usize::from(byte & 0x0f)]));
    }
    saida
}

#[cfg(test)]
mod tests {
    use super::hex_minusculo;
    use sha2::{Digest as _, Sha256};

    #[test]
    fn digitos_minusculos_com_zero_a_esquerda() {
        assert_eq!(hex_minusculo(&[0x00, 0x0a, 0xff, 0x10]), "000aff10");
        assert_eq!(hex_minusculo(&[]), "");
    }

    #[test]
    fn sha256_conhecido() {
        assert_eq!(
            hex_minusculo(&Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
