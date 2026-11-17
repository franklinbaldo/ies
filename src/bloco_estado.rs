//! O estado de um bloco como o SEI o publica na coluna "Estado" (#789).
//! O conjunto é fechado: um texto fora dele não vira estado. A saída JSON
//! traz o texto do SEI, como antes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoBloco {
    Concluido,
    Disponibilizado,
    Gerado,
    Recebido,
    Retornado,
}

impl EstadoDoBloco {
    const TODOS: [Self; 5] = [
        Self::Concluido,
        Self::Disponibilizado,
        Self::Gerado,
        Self::Recebido,
        Self::Retornado,
    ];

    /// O texto da coluna Estado como o SEI o publica.
    pub(crate) const fn como_no_sei(self) -> &'static str {
        match self {
            Self::Concluido => "Concluído",
            Self::Disponibilizado => "Disponibilizado",
            Self::Gerado => "Gerado",
            Self::Recebido => "Recebido",
            Self::Retornado => "Retornado",
        }
    }

    /// O estado cujo texto no SEI é exatamente `texto`.
    pub(crate) fn ler(texto: &str) -> Option<Self> {
        Self::TODOS
            .into_iter()
            .find(|estado| estado.como_no_sei() == texto)
    }
}

/// Na saída JSON, o estado aparece como o SEI o publica.
impl serde::Serialize for EstadoDoBloco {
    fn serialize<S: serde::Serializer>(&self, serializador: S) -> Result<S::Ok, S::Error> {
        serializador.serialize_str(self.como_no_sei())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_estado_vai_e_volta_pelo_texto_do_sei() {
        for estado in EstadoDoBloco::TODOS {
            assert_eq!(EstadoDoBloco::ler(estado.como_no_sei()), Some(estado));
        }
    }

    #[test]
    fn texto_fora_do_conjunto_nao_e_estado() {
        for texto in ["Enviado", "", "gerado", "Concluido"] {
            assert_eq!(EstadoDoBloco::ler(texto), None);
        }
    }

    #[test]
    fn json_traz_o_texto_do_sei() {
        assert_eq!(
            serde_json::to_string(&EstadoDoBloco::Concluido).unwrap(),
            "\"Concluído\""
        );
        assert_eq!(
            serde_json::to_string(&EstadoDoBloco::Gerado).unwrap(),
            "\"Gerado\""
        );
    }
}
