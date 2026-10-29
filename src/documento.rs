//! Identificador de documento informado pelo usuário, distinto do NUP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumeroDocumento(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumeroDocumentoInvalido {
    Vazio,
    NaoNumerico,
}

impl NumeroDocumentoInvalido {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::Vazio => "numero_documento_vazio",
            Self::NaoNumerico => "numero_documento_nao_numerico",
        }
    }
}

impl NumeroDocumento {
    pub fn novo(texto: &str) -> Result<Self, NumeroDocumentoInvalido> {
        let texto = texto.trim();
        if texto.is_empty() {
            return Err(NumeroDocumentoInvalido::Vazio);
        }
        if !texto.bytes().all(|b| b.is_ascii_digit()) {
            return Err(NumeroDocumentoInvalido::NaoNumerico);
        }
        Ok(Self(texto.to_owned()))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_zeros_e_documentos_grandes() {
        assert_eq!(NumeroDocumento::novo(" 00001 ").unwrap(), NumeroDocumento("00001".into()));
        assert!(NumeroDocumento::novo("123456789012345678901234567890").is_ok());
    }

    #[test]
    fn recusa_texto_e_numeros_unicode_por_tipo() {
        assert_eq!(
            NumeroDocumento::novo(" "),
            Err(NumeroDocumentoInvalido::Vazio)
        );
        assert_eq!(
            NumeroDocumento::novo("１２３"),
            Err(NumeroDocumentoInvalido::NaoNumerico)
        );
        assert_eq!(
            NumeroDocumento::novo("123-A"),
            Err(NumeroDocumentoInvalido::NaoNumerico)
        );
    }
}
