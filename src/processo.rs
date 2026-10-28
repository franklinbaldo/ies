//! Projeção de leitura detalhada de processo; independente da caixa e do transporte.
//! Não é uma implementação de consulta ao SEI.
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NumeroProcesso(String);

impl NumeroProcesso {
    pub fn novo(valor: &str) -> Result<Self, &'static str> {
        let valor = valor.trim();
        if valor.is_empty() || valor.len() > 80 || valor.chars().any(char::is_control) {
            return Err("numero de processo invalido");
        }
        Ok(Self(valor.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A situação é local à unidade, nunca um atributo global do processo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SituacaoUnidade {
    PendenteRecebimento,
    Aberto,
    Concluido,
    Desconhecido,
}

/// Não inventar identidade técnica de andamento a partir da ordem ou da data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Andamento {
    pub ocorrido_em: Option<String>,
    pub unidade: Option<String>,
    pub usuario: Option<String>,
    pub descricao: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtuacaoNaUnidade {
    pub unidade: String,
    pub situacao: SituacaoUnidade,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetalheProcesso {
    pub numero: NumeroProcesso,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_interno: Option<String>,
    pub atuacoes: Vec<AtuacaoNaUnidade>,
    pub andamentos: Vec<Andamento>,
}

impl DetalheProcesso {
    pub fn novo(numero: NumeroProcesso) -> Self {
        Self {
            numero,
            id_interno: None,
            atuacoes: Vec::new(),
            andamentos: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn situacoes_de_unidade_nao_sao_um_estado_global() {
        let mut processo = DetalheProcesso::novo(
            NumeroProcesso::novo("23106.000001/2026-01").unwrap(),
        );
        processo.atuacoes = vec![
            AtuacaoNaUnidade {
                unidade: "A".into(),
                situacao: SituacaoUnidade::Aberto,
            },
            AtuacaoNaUnidade {
                unidade: "B".into(),
                situacao: SituacaoUnidade::Concluido,
            },
            AtuacaoNaUnidade {
                unidade: "C".into(),
                situacao: SituacaoUnidade::PendenteRecebimento,
            },
        ];
        assert_eq!(processo.atuacoes.len(), 3);
        let saida = serde_json::to_value(&processo).unwrap();
        assert!(saida.get("aberto").is_none());
        assert!(saida.get("id_interno").is_none());
        assert_eq!(saida["atuacoes"][2]["situacao"], "pendente_recebimento");
    }

    #[test]
    fn andamento_sem_id_inventado() {
        let andamento = Andamento {
            ocorrido_em: None,
            unidade: Some("A".into()),
            usuario: None,
            descricao: "Processo encaminhado".into(),
        };
        let valor = serde_json::to_value(andamento).unwrap();
        assert!(valor.get("id").is_none());
        assert!(valor["ocorrido_em"].is_null());
    }

    #[test]
    fn numero_visivel_nao_exige_id_interno() {
        assert!(NumeroProcesso::novo(" ").is_err());
        assert!(NumeroProcesso::novo("ok\nsegredo").is_err());
        assert_eq!(
            NumeroProcesso::novo(" 23106.000001/2026-01 ").unwrap().as_str(),
            "23106.000001/2026-01"
        );
    }
}
