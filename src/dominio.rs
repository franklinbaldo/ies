//! Domínio da caixa do SEI: dados tipados, sem completude armazenada,
//! apresentação, origem da sessão, HTML ou links assinados.
use crate::erro_web::ErroCaixa;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct IdProcedimento(String);

impl IdProcedimento {
    pub fn novo(valor: &str) -> Result<Self, ErroCaixa> {
        if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroCaixa::IdInvalido);
        }
        Ok(Self(valor.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Processo {
    pub id: IdProcedimento,
    pub numero: String,
    pub visualizado: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Categoria {
    Recebidos,
    Gerados,
}

impl Categoria {
    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Recebidos => "recebidos",
            Self::Gerados => "gerados",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabela<T> {
    pub processos: Vec<T>,
    pub total_declarado: Option<usize>,
}

impl Tabela<Processo> {
    pub fn nova(
        categoria: Categoria,
        total_declarado: Option<usize>,
        processos: Vec<Processo>,
    ) -> Result<Self, ErroCaixa> {
        let mut ids = HashSet::new();
        for processo in &processos {
            if !ids.insert(processo.id.clone()) {
                return Err(ErroCaixa::IdRepetido {
                    categoria,
                    id: processo.id.as_str().to_owned(),
                });
            }
        }
        if total_declarado.is_some_and(|total| processos.len() > total) {
            return Err(ErroCaixa::TotalExcedido { categoria });
        }
        Ok(Self {
            processos,
            total_declarado,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caixa {
    pub recebidos: Tabela<Processo>,
    pub gerados: Tabela<Processo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identidade_e_numero_sao_dados_distintos() {
        assert!(IdProcedimento::novo("P42").is_err());
        assert!(IdProcedimento::novo("0042").is_ok());
        let p = Processo {
            id: IdProcedimento::novo("0042").unwrap(),
            numero: "23106.000001/2026-01".into(),
            visualizado: true,
        };
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"id":"0042","numero":"23106.000001/2026-01","visualizado":true}"#
        );
    }
    #[test]
    fn nenhuma_completude_e_armazenada() {
        let tabela = Tabela::nova(Categoria::Recebidos, Some(100), Vec::new()).unwrap();
        assert_eq!(tabela.total_declarado, Some(100));
        assert!(tabela.processos.is_empty());
    }
}
