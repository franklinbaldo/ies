//! Domínio da caixa do SEI: dados tipados, sem completude armazenada,
//! apresentação, origem da sessão, HTML ou links assinados.
use serde::Serialize;
use std::collections::HashSet;
use std::fmt;

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

/// Nunca inclui HTML, cookies, URLs ou conteúdo de processos nas mensagens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroCaixa {
    HtmlInvalido,
    TabelaAusente { categoria: Categoria },
    TabelaInvalida { categoria: Categoria },
    LegendaAusente { categoria: Categoria },
    IdInvalido,
    IdRepetido { categoria: Categoria, id: String },
    NumeroInvalido { categoria: Categoria, id: String },
    ProcessoAmbiguo { categoria: Categoria, id: String },
    TotalExcedido { categoria: Categoria },
    TotalDivergente { categoria: Categoria },
    FaixaInvalida { categoria: Categoria },
    FaixaDesconhecida { categoria: Categoria },
    FaixaAlterada { categoria: Categoria },
    FaixaDescontinua { categoria: Categoria },
    LimitePaginas,
    UnidadeInesperada,
    ContextoNaoVerificavel,
    ContextoDivergente,
    SemPaginas,
}

impl ErroCaixa {
    pub fn subcodigo(&self) -> &'static str {
        match self {
            Self::HtmlInvalido => "html_invalido",
            Self::TabelaAusente { .. } => "tabela_ausente",
            Self::TabelaInvalida { .. } => "tabela_invalida",
            Self::LegendaAusente { .. } => "legenda_ausente",
            Self::IdInvalido => "id_invalido",
            Self::IdRepetido { .. } => "id_repetido",
            Self::NumeroInvalido { .. } => "numero_invalido",
            Self::ProcessoAmbiguo { .. } => "processo_ambiguo",
            Self::TotalExcedido { .. } => "total_excedido",
            Self::TotalDivergente { .. } => "total_divergente",
            Self::FaixaInvalida { .. } => "faixa_invalida",
            Self::FaixaDesconhecida { .. } => "faixa_desconhecida",
            Self::FaixaAlterada { .. } => "faixa_alterada",
            Self::FaixaDescontinua { .. } => "faixa_descontinua",
            Self::LimitePaginas => "limite_paginas",
            Self::UnidadeInesperada => "unidade_inesperada",
            Self::ContextoNaoVerificavel => "contexto_nao_verificavel",
            Self::ContextoDivergente => "contexto_divergente",
            Self::SemPaginas => "sem_paginas",
        }
    }
    pub fn categoria(&self) -> Option<Categoria> {
        match self {
            Self::TabelaAusente { categoria }
            | Self::TabelaInvalida { categoria }
            | Self::LegendaAusente { categoria }
            | Self::IdRepetido { categoria, .. }
            | Self::NumeroInvalido { categoria, .. }
            | Self::ProcessoAmbiguo { categoria, .. }
            | Self::TotalExcedido { categoria }
            | Self::TotalDivergente { categoria }
            | Self::FaixaInvalida { categoria }
            | Self::FaixaDesconhecida { categoria }
            | Self::FaixaAlterada { categoria }
            | Self::FaixaDescontinua { categoria } => Some(*categoria),
            _ => None,
        }
    }
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::IdRepetido { id, .. }
            | Self::NumeroInvalido { id, .. }
            | Self::ProcessoAmbiguo { id, .. } => Some(id),
            _ => None,
        }
    }
}

impl fmt::Display for ErroCaixa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "leitura da caixa: {}", self.subcodigo())?;
        if let Some(categoria) = self.categoria() {
            write!(f, " ({})", categoria.rotulo())?;
        }
        if let Some(id) = self.id() {
            write!(f, " (id {id})")?;
        }
        Ok(())
    }
}
impl std::error::Error for ErroCaixa {}

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
