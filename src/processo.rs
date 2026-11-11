//! Projeção de leitura detalhada de processo; independente da caixa e do transporte.
//! Não é uma implementação de consulta ao SEI.
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NumeroProcesso(String);

/// Apenas entrada opaca: cada instalação decide a numeração que reconhece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, thiserror::Error)]
#[error("{self:?}")]
pub enum NumeroProcessoInvalido {
    Vazio,
    Controle,
    ReferenciaLegada,
}

impl NumeroProcesso {
    pub fn novo(valor: &str) -> Result<Self, NumeroProcessoInvalido> {
        // Validar a entrada bruta: trim removeria CR/LF/tab das extremidades.
        if valor.chars().any(char::is_control) {
            return Err(NumeroProcessoInvalido::Controle);
        }
        let valor = valor.trim();
        if valor.is_empty() {
            return Err(NumeroProcessoInvalido::Vazio);
        }
        if valor
            .get(..6)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("sei://"))
        {
            return Err(NumeroProcessoInvalido::ReferenciaLegada);
        }
        Ok(Self(valor.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Não inventar identidade técnica de andamento a partir da ordem ou da data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Andamento {
    pub ocorrido_em: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usuario: Option<String>,
    pub descricao: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetalheProcesso {
    pub numero: NumeroProcesso,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_interno: Option<String>,
    pub andamentos: Vec<Andamento>,
}

impl DetalheProcesso {
    pub fn novo(numero: NumeroProcesso) -> Self {
        Self {
            numero,
            id_interno: None,
            andamentos: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn andamento_sem_id_inventado() {
        let andamento = Andamento {
            ocorrido_em: "01/11/2026 10:00".into(),
            unidade: Some("A".into()),
            usuario: None,
            descricao: "Processo encaminhado".into(),
        };
        let valor = serde_json::to_value(andamento).unwrap();
        assert!(valor.get("id").is_none());
        assert_eq!(valor["ocorrido_em"], "01/11/2026 10:00");
        assert!(valor.get("usuario").is_none());
    }

    #[test]
    fn controles_no_inicio_ou_fim_nao_sao_apagados_pelo_trim() {
        for entrada in [
            "\n000123/2026",
            "000123/2026\n",
            "\t000123/2026",
            "000123/2026\r\n",
        ] {
            assert_eq!(
                NumeroProcesso::novo(entrada),
                Err(NumeroProcessoInvalido::Controle)
            );
        }
    }

    #[test]
    fn espacos_externos_sem_controles_continuam_aceitos() {
        assert_eq!(
            NumeroProcesso::novo(" 000123/2026 ").unwrap().as_str(),
            "000123/2026"
        );
        assert_eq!(
            NumeroProcesso::novo("  "),
            Err(NumeroProcessoInvalido::Vazio)
        );
    }

    #[test]
    fn numero_visivel_nao_exige_id_interno() {
        assert!(NumeroProcesso::novo(" ").is_err());
        assert!(NumeroProcesso::novo("ok\nsegredo").is_err());
        assert_eq!(
            NumeroProcesso::novo(" 23106.000001/2026-01 ")
                .unwrap()
                .as_str(),
            "23106.000001/2026-01"
        );
    }
}
