//! Propriedades de apresentação do protocolo SEI estritamente sintéticas.
//! Este valor não é sessão, credencial nem uma assinatura real.
//! Não persistir no SQLite: só o estado funcional pertence ao banco.

/// Token emulado estável entre páginas geradas pela mesma bancada.
/// Não fornece autenticação; existe apenas para a navegação da CLI.
pub(crate) const INFRA_HASH: &str = "4444444444444444444444444444444444444444444444444444444444444444";

#[cfg(test)]
mod tests {
    use super::INFRA_HASH;

    #[test]
    fn token_sintetico_tem_formato_hexadecimal_e_nao_e_credencial() {
        assert_eq!(INFRA_HASH.len(), 64);
        assert!(INFRA_HASH.bytes().all(|b| b.is_ascii_hexdigit()));
    }
}
